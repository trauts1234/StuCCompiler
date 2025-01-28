
pub enum Punctuation{
    OPENCURLY,CLOSECURLY,// ( )
    OPENSQUIGGLY,CLOSESQUIGGLY,// { }
    OPENSQUARE,CLOSESQUARE,// [ ]
    HASHTAG,DOUBLEHASHTAG//# ##
}

impl Punctuation {
    pub fn try_new(to_token: &str) -> Option<Punctuation> {
        match to_token {
            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),
            "[" => Some(Self::OPENSQUARE),
            "]" => Some(Self::CLOSESQUARE),
            "#" => Some(Self::HASHTAG),
            "##" => Some(Self::DOUBLEHASHTAG),
            _ => None
        }
    }
}