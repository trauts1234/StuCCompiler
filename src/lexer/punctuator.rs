#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    DASH,
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,
    AMPERSAND,
    OPENCURLY,
    CLOSECURLY,
    OPENSQUIGGLY,
    CLOSESQUIGGLY,
    OPENSQUARE,
    CLOSESQUARE,
    COMMA
}

impl Punctuator {
    pub fn try_new(to_token: &str) -> Option<Punctuator> {
        match to_token {
            "+" => Some(Self::PLUS),
            "-" => Some(Self::DASH),
            "*" => Some(Self::ASTERISK),
            "/" => Some(Self::FORWARDSLASH),
            "=" => Some(Self::EQUALS),
            ";" => Some(Self::SEMICOLON),
            "&" => Some(Self::AMPERSAND),

            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),
            "[" => Some(Self::OPENSQUARE),
            "]" => Some(Self::CLOSESQUARE),

            "," => Some(Self::COMMA),
            _ => None
        }
    }

     /**
     * if this punctuator can be a binary operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    pub fn as_binary_operator_precedence(&self) -> Option<i32> {

        match self {
            Self::PLUS => Some(2),
            Self::DASH => Some(2),
            Self::ASTERISK => Some(3),//binary operator as in multiply
            Self::FORWARDSLASH => Some(3),
            Self::EQUALS => Some(14),
            //TODO ampersand
            _ => None
        }
    }
    /**
     * if this punctuator can be a unary prefix operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    pub fn as_unary_prefix_precedence(&self) -> Option<i32> {
        match self {
            Self::ASTERISK => Some(2),
            Self::AMPERSAND => Some(2),
            _ => None
        }
    }
}
