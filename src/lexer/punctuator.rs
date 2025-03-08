#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    PLUSPLUS,

    DASH,
    
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,

    AMPERSAND,
    PERCENT,

    ANGLERIGHT,
    ANGLELEFT,
    LESSEQAUAL,
    GREATEREQUAL,
    DOUBLEEQUALS,

    OPENCURLY,
    CLOSECURLY,
    OPENSQUIGGLY,
    CLOSESQUIGGLY,
    OPENSQUARE,
    CLOSESQUARE,
    COMMA,

    ELIPSIS,
}

impl Punctuator {
    pub fn try_new(to_token: &str) -> Option<Punctuator> {
        match to_token {
            "+" => Some(Self::PLUS),
            "++" => Some(Self::PLUSPLUS),
            
            "-" => Some(Self::DASH),

            "*" => Some(Self::ASTERISK),
            "/" => Some(Self::FORWARDSLASH),
            "=" => Some(Self::EQUALS),
            ";" => Some(Self::SEMICOLON),

            "&" => Some(Self::AMPERSAND),
            "%" => Some(Self::PERCENT),

            ">" => Some(Self::ANGLERIGHT),
            "<" => Some(Self::ANGLELEFT),
            ">=" => Some(Self::GREATEREQUAL),
            "<=" => Some(Self::LESSEQAUAL),
            "==" =>Some(Self::DOUBLEEQUALS),

            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),
            "[" => Some(Self::OPENSQUARE),
            "]" => Some(Self::CLOSESQUARE),

            "," => Some(Self::COMMA),

            "..." => Some(Self::ELIPSIS),
            _ => None
        }
    }

    /**
     * if this punctuator is a comparison operator, what instruction would
     * returns the correct setcc instruction
     */
    pub fn as_comparator_instr(&self) -> Option<String> {
        match self {
            Self::ANGLELEFT => Some("setl"),
            Self::ANGLERIGHT => Some("setg"),
            Self::DOUBLEEQUALS => Some("sete"),
            Self::LESSEQAUAL => Some("setle"),
            Self::GREATEREQUAL => Some("setge"),
            _ => None,
        }.map(|x| x.to_string())
    }

     /**
     * if this punctuator can be a binary operator:
     * returns Some(precedence number)
     * if it can't: None
     */
    pub fn as_binary_operator_precedence(&self) -> Option<i32> {

        match self {
            Self::PLUS | Self::DASH => Some(4),
            Self::ASTERISK | Self::FORWARDSLASH | Self::PERCENT => Some(3),//binary operator as in multiply
            Self::EQUALS => Some(14),

            Self::ANGLELEFT | Self::ANGLERIGHT | Self::GREATEREQUAL | Self::LESSEQAUAL => Some(6),
            Self::DOUBLEEQUALS => Some(7),
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
            Self::ASTERISK => Some(2),//dereference
            Self::AMPERSAND => Some(2),//reference

            Self::PLUSPLUS => Some(2),

            Self::DASH => Some(2),//unary negate
            _ => None
        }
    }
}
