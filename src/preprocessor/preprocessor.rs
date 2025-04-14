use std::{fs, path::{Path, PathBuf}};

use regex::Regex;

use crate::preprocessor::{preprocess_boolean_operators::get_binary_numerical_text_and_functions, preprocess_context::ScanType};

use super::{preprocess_context::PreprocessContext, string_apply::Apply};

const INCLUDE_FOLDERS: &[&str] = &["c_lib"];//local custom version of glibc 

pub fn preprocess_c_file(filename: &Path) -> String {
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
    let multiline_comment_regex = Regex::new(r"/\*[\s\S]*?\*/").unwrap();
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

    let unsubstituted_line_trim = unsubstituted_line.trim_matches(|x: char| x == '\n' || x == ' ');
    //let substituted_line_trim = substituted_line.trim_matches(|x: char| x == '\n' || x == ' ');

    match unsubstituted_line_trim {//match on the raw line
        line if line.starts_with("#define") => {

            match ctx.get_scan_type() {
                ScanType::NORMAL => {
                    let mut split = unsubstituted_line_trim.split(" ");//raw line as substitutions don't happen in #define statements
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
                    let name = unsubstituted_line_trim.split_once(" ").unwrap().1;//raw line as substitutions don't happen in #undef statements
                    ctx.undefine(name);
                }
                _ => {}
            }
        }

        line if line.starts_with("#include") => {
            return manage_include_directive(include_limit, ctx, &unsubstituted_line);//no macros in #include statements
        }

        line if line.starts_with("#ifdef") => {

            let name = unsubstituted_line_trim.split_once(" ").unwrap().1; //no macros expanded in #ifdef, as I am looking for the macro name to see if it is defined
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
            let name = unsubstituted_line_trim.split_once(" ").unwrap().1;//no macros expanded in #ifndef, as I am looking for the macro name to see if it is defined
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
            let expr = unsubstituted_line_trim.split_once(" ").unwrap().1;
            let is_true = evaluate_const_expr(&expr.replace([' ', '\t', '\n'], ""), ctx) != 0;

            match ctx.get_scan_type() {
                ScanType::NORMAL if !is_true => {
                    ctx.set_scan_type(ScanType::FINDINGTRUEBRANCH(ctx.selection_depth()));//skip this branch as #if failed, so scan for else/elif etc.
                }
                _ => {}
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
    let mut next_char_is_escaped = false;
    
    for i in 0..line_of_file.len() {

        //set the preceeding character as a blank string if it is before the start of the string
        let preceeding_char = if i > 0 {&line_of_file[i-1..=i-1]} else {""};

        let remaining_str = &line_of_file[i..];

        match (preceeding_char, remaining_str) {
            ("\\", _) if !next_char_is_escaped => {
                next_char_is_escaped = true;//non-escaped backslash, will escape next character
                continue;//skip the code after the match
            }
            ("\"", _) if !next_char_is_escaped && !ctx.inside_char => {
                //non-escaped speechmark that isn't in a char
                ctx.inside_string ^= true;//flip inside string status
            },
            ("\'", _) if !next_char_is_escaped => {
                //non escaped single quote
                ctx.inside_char ^= true;//flip from in char to out, or vice versa
            },

            (curr, remaining) if !ctx.inside_string && !ctx.inside_char => {
                //not inside a string or char, try matching a #define

                //find the longest alphanumeric chain, as that will be matched
                //this also prevents matching: DEFINEDTEXTremainingvariablename as you cannot match in the middle of variable names etc.
                let longest_substitution: String = remaining.chars()
                    .take_while(is_identifier_token)
                    .collect();

                if !curr.chars().any(|arg0: char| is_identifier_token(&arg0))//ensure that the substitution is not immediately following an identifier
                    && ctx.is_defined(&longest_substitution) {//and there is a #define match
                    
                    let substitution = ctx.get_definition(&longest_substitution).unwrap();
                    let before_substitution = &line_of_file[..i];
                    let after_substitution = &line_of_file[(i+longest_substitution.len())..];
                    //put all the text before the match, then run the rest recursively, in case there are remaining substitutions or a substitution contains another substitution
                    //TODO stop the *same* substitution being used recursively on substitution?
                    return before_substitution.to_string() + &substitute_defines(ctx, &(substitution + after_substitution));
                }

            }

            (_,_) => {}
        }
        //made it to the end of the match, reset the escape
        next_char_is_escaped = false;
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

            let working_folder = find_first_working_path(INCLUDE_FOLDERS, &include_filename).expect("couldn't find a folder that had that header");
            let file_text = fs::read_to_string(working_folder).expect("found header file in folder, but couldn't open it");

            preprocess(include_limit, ctx, file_text)
        }
    }
}

/**
 * detects whether an expression in a #if preprocessor statement is true or not
 * note: needs spaces removed from expr first
 */
//TODO should this be deprecated in favour of constexpr_parsing.rs? how would that handle #if defined(x) and similar
fn evaluate_const_expr(expr: &str, ctx: &PreprocessContext) -> i64 {//or should it be i32??
    assert!(!ctx.inside_char && !ctx.inside_string);//#if commands are not in strings?
    assert!(!expr.contains(" ") && !expr.contains("\t") && !expr.contains("\n"));//whitespace not permitted. remove it first

    //if the expr is just a number, return it
    if let Ok(value) = expr.parse::<i64>() {
        return value;
    }

    let defined_command = "defined";
    if let Some(defined_idx) = expr.find(defined_command) {
        let defined_and_onwards = &expr[defined_idx+defined_command.len()..];
        let before_defined = &expr[..defined_idx];

        let (macro_name, remaining_text) = if defined_and_onwards.starts_with("(") {
            //#if defined(x)
            let closing_bracket_idx = defined_and_onwards.find(")").expect("failed to find closing bracket in #if defined(x) statement");
            let macro_name = defined_and_onwards[1..closing_bracket_idx].to_string();//skip (, consume until )
            let remaining = &defined_and_onwards[closing_bracket_idx+1..];//get after the )

            (macro_name, remaining)
        } else {
            //#if defined x
            let macro_name = match_identifier_str(&defined_and_onwards);
            let remaining = &defined_and_onwards[macro_name.len()..];

            (macro_name, remaining)
        };

        let macro_substitution = if ctx.is_defined(&macro_name) { "1" } else { "0" };

        let substituted_macro = format!("{}{}{}", before_defined, macro_substitution, remaining_text);//replaced the defined(x) with the result (1 or 0)

        return evaluate_const_expr(&substituted_macro, ctx);//recursively scan the rest of the macro
    }

    //code executed beyond this point has no defined(x) as that has been substituted already

    if let Some(last_open_bracket) = expr.rfind("("){
        //bracketed expression
        let before_bracket = &expr[..last_open_bracket];
        let in_and_after_bracket = &expr[last_open_bracket+1..];//contents of the bracket)remaining text
        let close_bracket_idx = in_and_after_bracket.find(")").expect("failed to find matching close bracket in #if expression");
        let in_bracket = &in_and_after_bracket[..close_bracket_idx];
        let after_bracket = &in_and_after_bracket[close_bracket_idx+1..];

        let bracket_substitution = evaluate_const_expr(in_bracket, ctx).to_string();

        return evaluate_const_expr(&format!("{}{}{}", before_bracket, bracket_substitution, after_bracket), ctx);
    }

    //todo ? : operator

    //this handles && || | & ^ operators
    for (binary_op_text, binary_op_function) in get_binary_numerical_text_and_functions() {
        if let Some(op_idx) = expr.rfind(binary_op_text) {//rfind as the associativity of the operators is left to right
            let before_op = &expr[..op_idx];
            let after_op = &expr[op_idx+binary_op_text.len()..];

            let lhs = evaluate_const_expr(before_op, ctx);
            let rhs = evaluate_const_expr(after_op, ctx);

            return binary_op_function(lhs, rhs);//take the function associated with this operator, and apply it to both sides
        }
    }

    //TODO test these
    match (expr.rfind("=="), expr.rfind("!=")) {
        (None, None) => {}//no equality operators to deal with

        (Some(eq_idx), y) if y.is_none_or(|neq_idx| neq_idx < eq_idx) => {//where the == comes last or is the only one
            let before_equals = &expr[..eq_idx];
            let after_equals = &expr[eq_idx+2..];

            let lhs = evaluate_const_expr(&before_equals, ctx);
            let rhs = evaluate_const_expr(&after_equals, ctx);

            return if lhs == rhs {1} else {0};
        },

        (x, Some(neq_idx)) if x.is_none_or(|eq_idx| eq_idx < neq_idx) => {//where != comes last or is the only one
            let before_neq = &expr[..neq_idx];
            let after_neq = &expr[neq_idx+2..];

            let lhs = evaluate_const_expr(&before_neq, ctx);
            let rhs = evaluate_const_expr(&after_neq, ctx);

            return if lhs != rhs {1} else {0};
        }

        _ => {panic!("unknown match arm when trying to resolve == and != in #if statement")}
    }

    //find furthest right relational >, <, >=, <=, and then use that
    //see preprocess_boolean_operators for help

    todo!()
}

fn match_identifier_str(expr: &str) -> String {
    expr.chars()
    .take_while(is_identifier_token)
    .collect()
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

fn is_identifier_token(c: &char) -> bool{
    c.is_alphanumeric() || *c == '_'//matches chars that could be part of an identifier
}