pub struct CString{
    unformatted_text: String//text as seen in the source code (incl "\n")
}

impl CString {
    pub fn try_new(to_token: &str) -> Option<CString> {
        if to_token.starts_with("\"") && to_token.ends_with("\"") {
            Some( CString {
                unformatted_text: to_token.to_string()
            })

        } else {
            None
        }
    }
}

