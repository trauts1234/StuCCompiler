use std::{fs, path::{Path, PathBuf}};

use regex::Regex;

use crate::{lexer::token::{self, Token}, preprocessor::{preprocess_boolean_operators::get_binary_numerical_text_and_functions, preprocess_context::ScanType, preprocess_token::PreprocessToken}};

use super::{preprocess_context::PreprocessContext, string_apply::Apply};

const INCLUDE_FOLDERS: &[&str] = &["c_lib"];//local custom version of glibc 
//TODO put these somewhere sensible
const DEFAULT_DEFINES: &str = 
"
#define __CHAR_BIT__      8
#define __SCHAR_MAX__     127
#define __SHRT_MAX__      32767
#define __INT_MAX__       2147483647
#define __LONG_MAX__      9223372036854775807L
#define __LONG_LONG_MAX__ 9223372036854775807LL
#define SCHAR_MAX   __SCHAR_MAX__
#define SHRT_MAX    __SHRT_MAX__
#define INT_MAX     __INT_MAX__
#define LONG_MAX    __LONG_MAX__
#define LLONG_MAX   __LONG_LONG_MAX__
#define SCHAR_MIN   (-__SCHAR_MAX__ - 1)
#define SHRT_MIN    (-__SHRT_MAX__ - 1)
#define INT_MIN     (-__INT_MAX__ - 1)
#define LONG_MIN    (-__LONG_MAX__ - 1L)
#define LLONG_MIN   (-__LONG_LONG_MAX__ - 1LL)
#define UCHAR_MAX   (__SCHAR_MAX__ * 2 + 1)
#define USHRT_MAX    (__SHRT_MAX__ * 2 + 1)
#define UINT_MAX     (__INT_MAX__ * 2U + 1U)
#define ULONG_MAX    (__LONG_MAX__ * 2UL + 1UL)
#define ULLONG_MAX   (__LONG_LONG_MAX__ * 2ULL + 1ULL)
#define __PTRDIFF_TYPE__ long int
#define __SIZE_TYPE__ long unsigned int";

pub fn preprocess_c_file(filename: &Path) -> Vec<Token> {
    let initial_tokens = read_tokenise(filename);

    let include_handled = handle_includes(initial_tokens, 10);

    handle_preprocessor_commands(include_handled)
}

fn read_tokenise(path: &Path) -> Vec<PreprocessToken> {
    let text = format!("{}\n{}\n",
        DEFAULT_DEFINES,
        fs::read_to_string(path).expect(&format!("failed to open file {:?}", path))
    )
    .replace("\r\n", "\n")//fix weird newlines
    .replace("\t", " ")//make all whitespace a space character or newline
    .replace("\\\n", "")//remove \ newline, a feature in c
    .apply(|x| remove_comments(x));//remove all comments

    PreprocessToken::parse(&text)
}

fn handle_includes(tokens: Vec<PreprocessToken>, include_limit: i32) -> Vec<PreprocessToken> {
    if include_limit == 0 {
        return tokens;//ran out of recursive depth, don't bother including anything else
    }

    tokens
    .into_iter()
    .flat_map(|tok| -> Box<dyn Iterator<Item = PreprocessToken>> {
        match tok {
            PreprocessToken::IncludeFile(path) => {
                let path = PathBuf::try_from(path).unwrap();
                Box::new(handle_includes(read_tokenise(&path), include_limit-1).into_iter())
            },
            PreprocessToken::IncludeLib(include_filename) => {
                let path = find_first_working_path(INCLUDE_FOLDERS, &include_filename).expect("couldn't find a folder that had that header");
                Box::new(handle_includes(read_tokenise(&path), include_limit-1).into_iter())
            },
            x => Box::new(std::iter::once(x)),
        }
    })
    .collect()
}

fn handle_preprocessor_commands(tokens: Vec<PreprocessToken>) -> Vec<Token> {
    let mut ctx = PreprocessContext::new();
    let mut result = Vec::new();

    for tok in tokens {
        match tok {
            PreprocessToken::IncludeLib(_) |
            PreprocessToken::IncludeFile(_) => panic!("you need to substitute includes before handling preprocessor commands"),

            PreprocessToken::Error(err) => {
                if ctx.get_scan_type() == ScanType::NORMAL {
                    panic!("encountered {}", err);
                }
            }

            PreprocessToken::IfDef(x) => {
                let defined = ctx.is_defined(&x);
                ctx.inc_selection_depth();
                if !defined && ctx.get_scan_type() == ScanType::NORMAL {
                    // Was previously scanning, but this conditional failed
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                }
            },
            PreprocessToken::IfNDef(x) => {
                let defined = ctx.is_defined(&x);
                ctx.inc_selection_depth();
                if defined && ctx.get_scan_type() == ScanType::NORMAL {
                    // Was previously scanning, but this conditional failed
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                }
            },

            PreprocessToken::If(condition_tokens) => {
                let condition: bool = panic!();
                ctx.inc_selection_depth();
                if !condition && ctx.get_scan_type() == ScanType::NORMAL {
                    // Was previously scanning, but this conditional failed
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                }
            },
            PreprocessToken::Pragma(_) => todo!(),
            PreprocessToken::Endif => {
                ctx.dec_selection_depth();

                match ctx.get_scan_type() {
                    //skipping in a previous scope, now i'm back to normal
                    ScanType::FINDINGTRUEBRANCH(dep) |
                    ScanType::SKIPPINGBRANCH(dep) if ctx.selection_depth() < dep => {
                        ctx.set_scan_type(ScanType::NORMAL);
                    }

                    // Normal, or skipping in an outer scope, so I don't have to change anything
                    _ => {}
                }
            },

            PreprocessToken::Else => {
                ctx.set_scan_type(match ctx.get_scan_type() {
                    ScanType::NORMAL => ScanType::SKIPPINGBRANCH(ctx.selection_depth()),//because I was in a taken branch, the else is not taken, so skip until out of it
                    ScanType::FINDINGTRUEBRANCH(dep) if dep == ctx.selection_depth() => ScanType::NORMAL,//because I was looking for a branch at the current level and else is a catch-all, take it
                    
                    x => x// if already skipping, continue to do so. if finding true branch in outer scope, leave it be
                })
            },
            PreprocessToken::Elif(condition_tokens) => {
                panic!()
            },
            PreprocessToken::DefineToken((name, value)) => {
                if ctx.get_scan_type() == ScanType::NORMAL {
                    ctx.define(name, value);
                }
            },
            PreprocessToken::DefineFunction(_) => todo!(),
            PreprocessToken::Undef(ident) => {
                if ctx.get_scan_type() == ScanType::NORMAL {
                    ctx.undefine(&ident);
                }
            }
            PreprocessToken::LineOfCode(line) => {
                if ctx.get_scan_type() == ScanType::NORMAL {
                    result.extend(
                        line.into_iter()
                        //handle defined macros in the code
                        .flat_map(|tok| {
                            //check if the token is an identifier
                            if let Token::IDENTIFIER(ref x) = tok {
                                //check if the identifier is bound to a macro
                                if let Some(definition) = ctx.get_definition(x) {
                                    definition//identifier was bound to macro, return it
                                } else {
                                    vec![tok]//identifier but not defined as a macro
                                }
                            } else {
                                vec![tok]//take the token
                            }
                        })
                    );//only add code if I want to read it
                }
            },
        }
    }

    result
}

/**
 * this replaces all comments in the text_file with " " as whitespace
 * TODO what about commenty-looking things inside or overlapping strings???!!!!
 */
pub fn remove_comments(text_file: String) -> String {
    let multiline_comment_regex = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
    let singleline_comment_regex = Regex::new(r"\/\/[^\n]*").unwrap();
    
    text_file
        .apply(|x| multiline_comment_regex.replace_all(&x, " ").to_string())//remove multiline comments
        .apply(|x| singleline_comment_regex.replace_all(&x, " ").to_string())//remove single line comments
}


fn find_first_working_path(folders: &[&str], filename: &str) -> Option<PathBuf> {
    for folder in folders {
        let path = Path::new(folder).join(filename);

        if path.exists() {
            return Some(path);
        }
    }

    None
}