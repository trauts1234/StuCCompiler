use std::{fs, path::{Path, PathBuf}, thread::panicking};

use regex::Regex;

use crate::preprocessor::preprocess_context::ScanType;

use super::{preprocess_context::PreprocessContext, string_apply::Apply};

pub fn preprocess_c_file(filename: &str) -> String {
    let file_text = fs::read_to_string(filename).expect("failed to open c file");
    preprocess(10, &mut PreprocessContext::new(), file_text)
}

/**
 * recursive function to preprocess a file
 * include_limit: how many times #include can be used recursively, to prevent:
 * a includes b, b includes a -> stack overflow
 */
fn preprocess(include_limit: i32, ctx: &mut PreprocessContext, file_text: String) -> String {
    
    file_text
    .replace("\r\n", "\n")//fix weird newlines
    .replace("\t", " ")//make all whitespace a space character or newline
    .replace("\\\n", "")//remove \ newline, a feature in c
    .apply(|x| remove_comments(x))//remove all comments
    .split("\n")//get each line
    .map(|x| parse_preprocessor(include_limit, ctx, x))//.scan(preprocessor_state, |state, ln| Some(parse_preprocessor(state, ln)))//apply the preprocessor to them
    .fold(String::new(), |acc, x| acc + &x)//join each line back together
}

/**
 * this replaces all comments in the text_file with " " as whitespace
 */
pub fn remove_comments(text_file: String) -> String {
    let multiline_comment_regex = Regex::new(r"/\*.*?\*/").unwrap();
    let singleline_comment_regex = Regex::new(r"\/\/[^\n]*").unwrap();
    
    text_file
        .apply(|x| multiline_comment_regex.replace_all(&x, " ").to_string())//remove multiline comments
        .apply(|x| singleline_comment_regex.replace_all(&x, " ").to_string())//remove single line comments
}

/**
 * note: this may return multiple lines, in the case of
 */
fn parse_preprocessor(include_limit: i32, ctx: &mut PreprocessContext, unsubstituted_line: &str) -> String {

    assert!(!ctx.inside_char);//no multiline chars??

    let skip_defines = ctx.inside_string;

    let substituted_line = substitute_defines(ctx, unsubstituted_line);

    if skip_defines && *ctx.get_scan_type() == ScanType::NORMAL {
        return substituted_line;//this line started as being inside a string, so can't possibly be any preprocessor directive
    }

    match substituted_line.trim_matches(|x: char| x == '\n' || x == ' ') {
        line if line.starts_with("#define") => {

            match ctx.get_scan_type() {
                ScanType::NORMAL => {
                    let mut split = line.split(" ");
                    split.next();//consume the #define

                    let name = split.next().expect("tried to get name from #define but couldn't find it");
                    let value = split.next().or(Some("")).unwrap();// set what it is defined to, or an empty string

                    ctx.define(name,value);
                }
                _ => {}//other: needs skipping
            }
        }
        
        line if line.starts_with("#undef") => {
            match ctx.get_scan_type() {
                ScanType::NORMAL => {
                    let name = line.split_once(" ").unwrap().1;
                    ctx.undefine(name);
                }
                _ => {}
            }
        }

        line if line.starts_with("#include") => {
            return manage_include_directive(include_limit, ctx, &substituted_line);
        }

        line if line.starts_with("#ifdef") => {

            let name = line.split_once(" ").unwrap().1;
            let defined = ctx.is_defined(name);

            match ctx.get_scan_type() {
                ScanType::NORMAL if !defined => {//reading the file normally and this macro is not defined
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));//was scanning normally but ifdef failed, so start finding a subsequent true branch
                }
                _ => {}
            }

            ctx.inc_selection_depth();
        }

        line if line.starts_with("#ifndef") => {
            let name = line.split_once(" ").unwrap().1;
            let defined = ctx.is_defined(name);

            match ctx.get_scan_type() {
                ScanType::NORMAL if defined => {//reading the file normally and this macro was defined
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));//was scanning normally but ifndef failed, so start finding a subsequent true branch
                }
                _ => {}
            }

            ctx.inc_selection_depth();
        }

        //this one should be lower as #if* matches #ifdef and others
        line if line.starts_with("#if") => {
            let expr = line.split_once(" ").unwrap().1;
            let is_true = ctx.is_expr_true(expr);

            match ctx.get_scan_type() {
                ScanType::NORMAL if is_true => {
                    todo!()
                }
                _ => {todo!()}
            }
        }

        line if line.starts_with("#else") => {
            match ctx.get_scan_type() {
                ScanType::NORMAL => {
                    ctx.set_scan_type(ScanType::SKIPPINGBRANCH(ctx.selection_depth() - 1));//the previous branch was taken, so this else must be skipped (note selection depth minus one)
                }
                ScanType::FINDINGTRUEBRANCH(depth) if ctx.selection_depth() - 1 == *depth => {//found an else that I should take in the correct level
                    ctx.set_scan_type(ScanType::NORMAL);
                }
                _ => {}
            }
        }

        line if line.starts_with("#elif") => {
            todo!()
        }

        line if line.starts_with("#endif") => {
            ctx.dec_selection_depth();//all scan types require decrementing the branch counter

            match ctx.get_scan_type() {
                ScanType::SKIPPINGBRANCH(depth) if ctx.selection_depth() <= *depth => {
                    ctx.set_scan_type(ScanType::NORMAL);//reached end of an if statement I was skipping
                },
                ScanType::FINDINGTRUEBRANCH(depth) if ctx.selection_depth() <= *depth => {
                    ctx.set_scan_type(ScanType::NORMAL);//looking for true branch, but reached end of if statement
                }

                _ => {}//other situations, do nothing
            }
        }

        line if line.starts_with("#error") && *ctx.get_scan_type() == ScanType::NORMAL => {
            panic!("found a #error token in the preprocessor");
        }

        _ => {
            match ctx.get_scan_type() {
                ScanType::NORMAL => {return substituted_line.to_string()},//no preprocessor, and I am reading file lines normally
                _ => {}
            }
        }

    }

    String::new()

    
}

/**
 * takes in a line of a file, and substitutes everything defined by #define
 */
fn substitute_defines(ctx: &mut PreprocessContext, line_of_file: &str) -> String {
    for i in 0..line_of_file.len() {

        //set the preceeding character as a blank string if it is before the start of the string
        let preceeding_char = if i > 0 {&line_of_file[i-1..=i-1]} else {""};

        let remaining_str = &line_of_file[i..];

        match (preceeding_char, remaining_str) {
            (curr, remaining) if curr != "\\" && remaining.starts_with("\"") && !ctx.inside_char => {
                //non-escaped speechmark that isn't in a char
                ctx.inside_string ^= true;//flip inside string status
            },
            (curr, remaining) if curr != "\\" && remaining.starts_with("'") => {
                //non escaped single quote
                ctx.inside_char ^= true;//flip from in char to out, or vice versa
            },

            (curr, remaining) if !ctx.inside_string && !ctx.inside_char => {
                //not inside a string or char, try matching a #define

                let is_identifier_token = |c: &char| c.is_alphanumeric() || *c == '_';//matches chars that could be part of an identifier

                //find the longest alphanumeric chain, as that will be matched
                //this also prevents matching: DEFINEDTEXTremainingvariablename as you cannot match in the middle of variable names etc.
                let longest_substitution: String = remaining.chars()
                    .take_while(is_identifier_token)
                    .collect();

                if !curr.chars().any(|arg0: char| is_identifier_token(&arg0))//ensure that the substitution is not immediately following an identifier
                    && ctx.is_defined(&longest_substitution) {//and there is a #define match
                    
                    let substitution = ctx.get_definition(&longest_substitution).unwrap();
                    let before_substitution = &line_of_file[..=i];
                    let after_substitution = &line_of_file[(i+longest_substitution.len())..];
                    //put all the text before the match, then run the rest recursively, in case there are remaining substitutions or a substitution contains another substitution
                    //TODO stop the *same* substitution being used recursively on substitution?
                    return before_substitution.to_string() + &substitute_defines(ctx, &(substitution + after_substitution));
                }

            }

            (_,_) => {}
        }
    }
    line_of_file.to_string() //no match found, return whole line as is
}


fn manage_include_directive(include_limit: i32, ctx: &mut PreprocessContext, line: &str) -> String {
    assert!(line.starts_with("#include"));
    let potential_speechmark_start = line.find("\"");
    let potential_anglebracket_start = line.find("<");

    match (potential_speechmark_start, potential_anglebracket_start) {
        (None, None) => panic!("found #include directive but no speech mark or < in line:\n{}", line),
        (Some(_), Some(_)) => panic!("can't decide whether there is a speech mark or < include directive in line:\n{}", line),
        (Some(_speechmark_start), None) => {
            let include_filename = line.split("\"").nth(1).unwrap();// #include "foo.h", the filename is the second item in the string after split by "

            let file_text = fs::read_to_string(include_filename).expect("failed to open header file in local directory");
            preprocess(include_limit, ctx, file_text)//include limit is already decremented
        }
        (None, Some(anglebracket_start)) => {
            let close_bracket = line.find(">").expect("can't find close bracket in #include");

            let include_filename = &line[anglebracket_start + 1..close_bracket];
            let include_folders = vec!["/usr/include", "/usr/local/include", "/usr/include/x86_64-linux-gnu"];

            let working_folder = find_first_working_path(include_folders, &include_filename).expect("couldn't find a folder that had that header");
            let file_text = fs::read_to_string(working_folder).expect("found header file in folder, but couldn't open it");

            preprocess(include_limit, ctx, file_text)
        }
    }
}

fn find_first_working_path(folders: Vec<&str>, filename: &str) -> Option<PathBuf> {
    for folder in folders {
        let path = Path::new(folder).join(filename);

        if path.exists() {
            return Some(path);
        }
    }

    None
}