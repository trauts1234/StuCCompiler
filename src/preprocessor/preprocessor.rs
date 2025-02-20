use std::fs;

use regex::Regex;

use crate::{compilation_error::CompilationError, preprocessor::preprocess_context::ScanType};

use super::{preprocess_context::PreprocessContext, string_apply::Apply};

const INCLUDE_FOLDER: &str = "/usr/include";

pub fn preprocess_c_file(filename: &str) -> Result<String, CompilationError> {
    preprocess(10, &mut PreprocessContext::new(), filename)
}

/**
 * recursive function to preprocess a file
 * include_limit: how many times #include can be used recursively, to prevent:
 * a includes b, b includes a -> stack overflow
 */
fn preprocess(include_limit: i32, ctx: &mut PreprocessContext, filename: &str) -> Result<String, CompilationError> {
    let file_text = fs::read_to_string(filename)?;
    
    Ok(file_text
        .replace("\r\n", "\n")//fix weird newlines
        .replace("\t", " ")//make all whitespace a space character or newline
        .replace("\\\n", "")//remove \ newline, a feature in c
        .apply(|x| remove_comments(x))//remove all comments
        .split("\n")//get each line
        .map(|x| parse_preprocessor(include_limit, ctx, x))//.scan(preprocessor_state, |state, ln| Some(parse_preprocessor(state, ln)))//apply the preprocessor to them
        .collect::<Result<Vec<_>,_>>()?//propogate any errors
        .iter()
        .fold(String::new(), |acc, x| acc + &x)//join each line back together
    )
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
fn parse_preprocessor(include_limit: i32, ctx: &mut PreprocessContext, line_of_file: &str) -> Result<String, CompilationError> {

    if line_of_file == "#endif"{
        println!("found endif");
    }
    match line_of_file.trim_matches(|x: char| x == '\n' || x == ' ') {
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
            return manage_include_directive(include_limit, ctx, line_of_file);
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
                    ctx.set_scan_type(ScanType::SKIPPINGBRANCH(ctx.selection_depth()));//the previous branch was taken, so this else must be skipped
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
                ScanType::SKIPPINGBRANCH(x) => {println!("{}", x);}
                ScanType::SKIPPINGBRANCH(depth) if ctx.selection_depth() == *depth => {
                    ctx.set_scan_type(ScanType::NORMAL);//reached end of an if statement I was skipping
                },
                ScanType::FINDINGTRUEBRANCH(depth) if ctx.selection_depth() == *depth => {
                    ctx.set_scan_type(ScanType::NORMAL);//looking for true branch, but reached end of if statement
                }

                _ => {}//other situations, do nothing
            }
        }

        line if line.starts_with("#error") && *ctx.get_scan_type() == ScanType::NORMAL => {
            return Err(CompilationError::HASHERR);
        }

        _ => {
            match ctx.get_scan_type() {
                ScanType::NORMAL => {return Ok(line_of_file.to_string())},//no preprocessor, and I am reading file lines normally
                _ => {}
            }
        }

    }

    Ok(String::new())

    
}


fn manage_include_directive(include_limit: i32, ctx: &mut PreprocessContext, line: &str) -> Result<String, CompilationError>{
    assert!(line.starts_with("#include"));
    let potential_speechmark_start = line.find("\"");
    let potential_anglebracket_start = line.find("<");

    match (potential_speechmark_start, potential_anglebracket_start) {
        (None, None) => panic!("found #include directive but no speech mark or < in line:\n{}", line),
        (Some(_), Some(_)) => panic!("can't decide whether there is a speech mark or < include directive in line:\n{}", line),
        (Some(_speechmark_start), None) => {
            let include_filename = line.split("\"").nth(1).unwrap();// #include "foo.h", the filename is the second item in the string after split by "
            preprocess(include_limit, ctx, include_filename)//include limit is already decremented
        }
        (None, Some(anglebracket_start)) => {
            let close_bracket = line.find(">").ok_or(CompilationError::MISC("can't find close bracket in #include".to_string()))?;
            let include_filename = format!("{}/{}.h", INCLUDE_FOLDER, &line[anglebracket_start + 1..close_bracket]);
            preprocess(include_limit, ctx, &include_filename)
        }
    }
}