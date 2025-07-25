use std::{fs, path::{Path, PathBuf}};

use crate::{lexer::token::Token, number_literal::typed_value::NumberLiteral, preprocessor::{preprocess_constant_fold::{fold, is_true, sub_definitions}, preprocess_context::ScanType, preprocess_token::{LineNumbered, PreprocessLine, PreprocessToken}}};

use super::preprocess_context::PreprocessContext;

const INCLUDE_FOLDERS: &[&str] = &["c_lib"];//local custom version of glibc 

pub fn preprocess_c_file(filename: &Path) -> Vec<Token> {
    let initial_tokens = read_tokenise(filename);

    let include_handled = handle_includes(initial_tokens, 10);

    handle_preprocessor_commands(include_handled, filename.file_name().unwrap().to_str().unwrap())
}

fn read_tokenise(path: &Path) -> Vec<LineNumbered> {
    let text = format!("{}\n",
        fs::read_to_string(path).expect(&format!("failed to open file {:?}", path))
    )
    .replace("\r\n", "\n")//fix weird newlines
    .replace("\t", " ")//make all whitespace a space character or newline
    .replace("\\\n", "");//remove \ newline, a feature in c

    let text = remove_comments(&text.chars().collect::<Vec<_>>()[..]);

    PreprocessToken::parse(&text)
}

fn handle_includes(tokens: Vec<LineNumbered>, include_limit: i32) -> Vec<LineNumbered> {
    if include_limit == 0 {
        return tokens;//ran out of recursive depth, don't bother including anything else
    }

    tokens
    .into_iter()
    .flat_map(|tok| -> Box<dyn Iterator<Item = LineNumbered>> {
        match tok.data {
            PreprocessToken::Preprocessor(PreprocessLine::IncludeFile(path)) => {
                let path = PathBuf::try_from(path).unwrap();
                Box::new(handle_includes(read_tokenise(&path), include_limit-1).into_iter())
            },
            PreprocessToken::Preprocessor(PreprocessLine::IncludeLib(include_filename)) => {
                let path = find_first_working_path(INCLUDE_FOLDERS, &include_filename).expect("couldn't find a folder that had that header");
                Box::new(handle_includes(read_tokenise(&path), include_limit-1).into_iter())
            },
            _ => Box::new(std::iter::once(tok)),
        }
    })
    .collect()
}

fn handle_preprocessor_commands(tokens: Vec<LineNumbered>, filename: &str) -> Vec<Token> {
    let mut ctx = PreprocessContext::new(filename);
    let mut result = Vec::new();

    for tok in tokens {
        ctx.set_line_number(tok.line_num);
        match tok.data {
            PreprocessToken::Preprocessor(preprocess) => match preprocess {
                PreprocessLine::NullDirective => {},//this does nothing
                PreprocessLine::IncludeLib(_) |
                PreprocessLine::IncludeFile(_) => panic!("you need to substitute includes before handling preprocessor commands"),

                PreprocessLine::LineDirective((new_line, new_file)) => {
                    match new_line {
                        NumberLiteral::INTEGER { data, .. } => ctx.override_line_number(data.try_into().unwrap()),
                        _ => panic!("found float when trying to set line number with #line")
                    }
                    if let Some(new_filename) = new_file {
                        println!("new filename {:?}", new_filename);
                        ctx.override_filename(new_filename);
                    }
                }

                PreprocessLine::Error(err) => {
                    if ctx.get_scan_type() == ScanType::NORMAL {
                        panic!("encountered {}", err);
                    }
                }

                PreprocessLine::IfDef(x) => {
                    let defined = ctx.is_defined(&x);
                    ctx.inc_selection_depth();
                    if !defined && ctx.get_scan_type() == ScanType::NORMAL {
                        // Was previously scanning, but this conditional failed
                        ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                    }
                },
                PreprocessLine::IfNDef(x) => {
                    let defined = ctx.is_defined(&x);
                    ctx.inc_selection_depth();
                    if defined && ctx.get_scan_type() == ScanType::NORMAL {
                        // Was previously scanning, but this conditional failed
                        ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                    }
                },

                PreprocessLine::If(condition_tokens) => {
                    let condition: bool = is_true(fold(condition_tokens, &ctx));
                    ctx.inc_selection_depth();
                    if !condition && ctx.get_scan_type() == ScanType::NORMAL {
                        // Was previously scanning, but this conditional failed
                        ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));
                    }
                },
                PreprocessLine::Pragma(_) => todo!(),
                PreprocessLine::Endif => {
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

                PreprocessLine::Else => {
                    ctx.set_scan_type(match ctx.get_scan_type() {
                        ScanType::NORMAL => ScanType::SKIPPINGBRANCH(ctx.selection_depth()),//because I was in a taken branch, the else is not taken, so skip until out of it
                        ScanType::FINDINGTRUEBRANCH(dep) if dep == ctx.selection_depth() => ScanType::NORMAL,//because I was looking for a branch at the current level and else is a catch-all, take it
                        
                        x => x// if already skipping, continue to do so. if finding true branch in outer scope, leave it be
                    })
                },
                PreprocessLine::Elif(condition_tokens) => {
                    let condition: bool = is_true(fold(condition_tokens, &ctx));
                    match ctx.get_scan_type() {
                        ScanType::NORMAL => {
                            //was previously on taken branch, now skip all branches at this depth
                            ctx.set_scan_type(ScanType::SKIPPINGBRANCH(ctx.selection_depth()));
                        }

                        ScanType::FINDINGTRUEBRANCH(dep) if dep == ctx.selection_depth() && condition => {
                            // Was previously looking for a true branch, and this one is it
                            ctx.set_scan_type(ScanType::NORMAL);
                        }

                        _ => {}//either continue trying to find a true branch or continue skipping depending on conditions
                    }
                },
                PreprocessLine::DefineToken((name, value)) => {
                    if ctx.get_scan_type() == ScanType::NORMAL {
                        ctx.define(name, value);
                    }
                },
                PreprocessLine::Undef(ident) => {
                    if ctx.get_scan_type() == ScanType::NORMAL {
                        ctx.undefine(&ident);
                    }
                }
            }
            
            PreprocessToken::LineOfCode(line) => {
                if ctx.get_scan_type() == ScanType::NORMAL {
                    result.extend(sub_definitions(line, &ctx, &Vec::new()));
                }
            },
        }
    }

    result
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

fn remove_comments(data: &[char]) -> String {
    enum State {
        Normal,
        CharLit,
        StringLit,
        LineComment,
        MultilineComment,
    }
    let mut state = State::Normal;
    let mut result = String::new();

    for i in 0..data.len() {
        let curr = data[i];
        let prev = if let Some(prev_idx) = i.checked_sub(1) {Some(data[prev_idx])} else {None};

        match (&state, prev, curr) {
            (State::Normal, _, '"') => {
                state = State::StringLit//start of string literal
            }
            (State::Normal, _, '\'') => {
                state = State::CharLit//start of char literal
            }
            (State::Normal, Some('/'), '/') => {
                state = State::LineComment;// start of single line comment
                assert_eq!(result.pop(), Some('/'));//remove the first / that was accidentally added
                continue;
            }
            (State::Normal, Some('/'), '*') => {
                state = State::MultilineComment;//start of multiline comment
                assert_eq!(result.pop(), Some('/'));//remove the first / that was accidentally added
                continue;
            }
            (State::Normal, _, _) => {}//normal character

            (State::LineComment, _, '\n') => {
                state = State::Normal//end of single line comment - keep the newline though
            }
            (State::LineComment, _, _) => continue,//skip character in comment

            (State::MultilineComment, Some('*'), '/') => {
                state = State::Normal;//end of multiline comment
                result.push(' ');//push whitespace to ensure that multiline comment becomes a whitespace character of some sort
                continue;//don't push the '/'
            }
            (State::MultilineComment, _, '\n') => {}//newlines in multiline comments are preserved - note that this causes problems
            (State::MultilineComment, _, _) => continue,//skip character in comment

            (State::CharLit, Some('\\'), _) => {}//escaped charcacter, take it
            (State::CharLit, _, '\'') => {
                state = State::Normal//not escaped quote mark ends char literal
            }
            (State::CharLit, _, _) => {}//other character, take it

            (State::StringLit, Some('\\'), _) => {}//escaped character, take it
            (State::StringLit, _, '"') => {
                state = State::Normal//not escaped speech mark ends string literal
            }
            (State::StringLit, _, _) => {}//other character, take it
        }

        //match done, push the character
        result.push(curr);
    }

    result
}