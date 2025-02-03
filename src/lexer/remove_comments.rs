use regex::Regex;

pub fn remove_comments(text_file: &str) -> String {
    let multiline_comment_regex = Regex::new(r"/\*.*?\*/").unwrap();
    let singleline_comment_regex = Regex::new(r"\/\/[^\n]*").unwrap();
    
    text_file
        .replace("\r\n", "\n")//fix weird newlines
        .replace("\t", " ")//make all whitespace a space character or newline
        .replace("\\\n", "")//remove \ newline, a feature in c
        .apply(|x| multiline_comment_regex.replace_all(&x, " ").to_string())//remove multiline comments
        .apply(|x| singleline_comment_regex.replace_all(&x, " ").to_string())//remove single line comments
}

//add string trait to apply a function to it
trait Apply {
    fn apply<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
        Self: Sized;
}

impl Apply for String {
    fn apply<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }
}