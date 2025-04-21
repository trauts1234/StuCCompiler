#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    PLUSPLUS,

    DASH,
    DASHDASH,
    
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,
    Tilde,

    Pipe,
    PIPEPIPE,
    ANDAND,

    Hat,
    AMPERSAND,
    PERCENT,
    Exclamation,

    Greater,
    GreaterGreater,
    Less,
    LessLess,
    LESSEQUAL,
    GREATEREQUAL,
    DOUBLEEQUALS,
    EXCLAMATIONEQUALS,

    OPENCURLY,
    CLOSECURLY,
    OPENSQUIGGLY,
    CLOSESQUIGGLY,
    OPENSQUARE,
    CLOSESQUARE,
    COMMA,

    FULLSTOP,
    ELIPSIS,
}

impl Punctuator {
    pub fn try_new(to_token: &str) -> Option<Punctuator> {
        match to_token {
            "+" => Some(Self::PLUS),
            "++" => Some(Self::PLUSPLUS),
            
            "-" => Some(Self::DASH),
            "--" => Some(Self::DASHDASH),

            "*" => Some(Self::ASTERISK),
            "/" => Some(Self::FORWARDSLASH),
            "=" => Some(Self::EQUALS),
            ";" => Some(Self::SEMICOLON),

            "%" => Some(Self::PERCENT),
            "^" => Some(Self::Hat),
            "!" => Some(Self::Exclamation),
            "~" => Some(Self::Tilde),

            "|" => Some(Self::Pipe),
            "||" => Some(Self::PIPEPIPE),
            "&" => Some(Self::AMPERSAND),
            "&&" => Some(Self::ANDAND),

            ">" => Some(Self::Greater),
            ">>" => Some(Self::GreaterGreater),
            "<" => Some(Self::Less),
            "<<" => Some(Self::LessLess),
            ">=" => Some(Self::GREATEREQUAL),
            "<=" => Some(Self::LESSEQUAL),
            "==" => Some(Self::DOUBLEEQUALS),
            "!=" => Some(Self::EXCLAMATIONEQUALS),

            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),
            "[" => Some(Self::OPENSQUARE),
            "]" => Some(Self::CLOSESQUARE),

            "," => Some(Self::COMMA),

            "." => Some(Self::FULLSTOP),
            "..." => Some(Self::ELIPSIS),
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


            Self::ASTERISK | Self::FORWARDSLASH | Self::PERCENT => Some(3),//binary operator as in multiply
            Self::PLUS | Self::DASH => Some(4),
            Self::LessLess | Self::GreaterGreater => Some(5),//bitwise shifts
            Self::Less | Self::Greater | Self::GREATEREQUAL | Self::LESSEQUAL => Some(6),
            Self::DOUBLEEQUALS | Self::EXCLAMATIONEQUALS => Some(7),
            Self::AMPERSAND => Some(8),//bitwise and
            Self::Hat => Some(9),
            Self::Pipe => Some(10),
            Self::ANDAND => Some(11),
            Self::PIPEPIPE => Some(12),

            Self::EQUALS => Some(14),
            _ => None
        }
    }
    /**
     * if this punctuator can be a unary prefix operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    //TODO maybe mobe to expression operator themselves
    pub fn as_unary_prefix_precedence(&self) -> Option<i32> {
        match self {
            Self::ASTERISK |//dereference
            Self::AMPERSAND |//reference
            Self::Exclamation |//boolean not
            Self::Tilde |
            Self::PLUSPLUS |
            Self::DASHDASH |//prefix increment/decrement
            Self::PLUS |//unary plus
            Self::DASH => Some(2),//unary negate
            _ => None
        }
    }
}

impl<'a> Into<&'a str> for Punctuator {
    fn into(self) -> &'a str {
        match self {
            Self::PLUS => "+",
            Self::PLUSPLUS => "++",
            Self::DASH => "-",
            Self::DASHDASH => "--",
            Self::ASTERISK => "*",
            Self::FORWARDSLASH => "/",
            Self::EQUALS => "=",
            Self::SEMICOLON => ";",
            Self::Tilde => "~",
            Self::Pipe => "|",
            Self::PIPEPIPE => "||",
            Self::ANDAND => "&&",
            Self::Hat => "^",
            Self::AMPERSAND => "&",
            Self::PERCENT => "%",
            Self::Exclamation => "!",
            Self::Greater => ">",
            Self::GreaterGreater => ">>",
            Self::Less => "<",
            Self::LessLess => "<<",
            Self::LESSEQUAL => "<=",
            Self::GREATEREQUAL => ">=",
            Self::DOUBLEEQUALS => "==",
            Self::EXCLAMATIONEQUALS => "!=",
            Self::OPENCURLY => "(",
            Self::CLOSECURLY => ")",
            Self::OPENSQUIGGLY => "{",
            Self::CLOSESQUIGGLY => "}",
            Self::OPENSQUARE => "[",
            Self::CLOSESQUARE => "]",
            Self::COMMA => ",",
            Self::FULLSTOP => ".",
            Self::ELIPSIS => "...",
        }
    }
}
