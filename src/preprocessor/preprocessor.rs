use std::fs;

use regex::Regex;

use crate::compilation_error::CompilationError;

use super::string_apply::Apply;

const INCLUDE_FOLDER: &str = "/usr/include";

pub fn preprocess(include_limit: i32, filename: &str) -> Result<String, CompilationError> {
    let file_text = fs::read_to_string(filename)?;
    
    Ok(file_text
        .replace("\r\n", "\n")//fix weird newlines
        .replace("\t", " ")//make all whitespace a space character or newline
        .replace("\\\n", "")//remove \ newline, a feature in c
        .apply(|x| remove_comments(x))//remove all comments
        .split("\n")//get each line
        .map(|x| parse_preprocessor(include_limit, x))//.scan(preprocessor_state, |state, ln| Some(parse_preprocessor(state, ln)))//apply the preprocessor to them
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
fn parse_preprocessor(include_limit: i32, line_of_file: &str) -> Result<String, CompilationError> {
    match line_of_file.trim_matches(|x: char| x == '\n' || x == ' ') {
        line if line.starts_with("#include") => {
            if include_limit > 0 {
                manage_include_directive(include_limit-1, line)
            } else {
                println!("#include depth reached, ignoring line:\n{}", line);
                Ok(String::new())//ignore include as it is probably infinitely recursive
            }
        },

        line if line.starts_with("#ifdef") => {
            todo!()
        },

        line if line.starts_with("#error") => {
            Err(CompilationError::HASHERR)
        },

        line if line.starts_with("#warning") => {
            println!("warning encountered with message: {}", line.split_at(8).1);
            Ok(String::new())//remove this line
        },

        line => Ok(line.to_string())//normal line of code, use it
    }
}

fn manage_include_directive(include_limit: i32, line: &str) -> Result<String, CompilationError>{
    assert!(line.starts_with("#include"));
    let potential_speechmark_start = line.find("\"");
    let potential_anglebracket_start = line.find("<");

    match (potential_speechmark_start, potential_anglebracket_start) {
        (None, None) => panic!("found #include directive but no speech mark or < in line:\n{}", line),
        (Some(_), Some(_)) => panic!("can't decide whether there is a speech mark or < include directive in line:\n{}", line),
        (Some(_speechmark_start), None) => {
            let include_filename = line.split("\"").nth(1).unwrap();// #include "foo.h", the filename is the second item in the string after split by "
            preprocess(include_limit, include_filename)//include limit is already decremented
        }
        (None, Some(anglebracket_start)) => {
            let close_bracket = line.find(">").ok_or(CompilationError::MISC("can't find close bracket in #include".to_string()))?;
            let include_filename = format!("{}/{}.h", INCLUDE_FOLDER, &line[anglebracket_start + 1..close_bracket]);
            preprocess(include_limit, &include_filename)
        }
    }
}