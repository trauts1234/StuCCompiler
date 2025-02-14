
#[derive(Debug, Clone, PartialEq)]
pub enum MathematicalOperator {
    ADD,
    SUBTRACT,
    MULTIPLY,
    DIVIDE,
    ASSIGN,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Punctuator {
    PLUS,
    DASH,
    ASTERISK,
    FORWARDSLASH,
    EQUALS,
    SEMICOLON,
    OPENCURLY,
    CLOSECURLY,
    OPENSQUIGGLY,
    CLOSESQUIGGLY,
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

            "(" => Some(Self::OPENCURLY),
            ")" => Some(Self::CLOSECURLY),
            "{" => Some(Self::OPENSQUIGGLY),
            "}" => Some(Self::CLOSESQUIGGLY),

            "," => Some(Self::COMMA),
            _ => None
        }
    }

    pub fn as_mathematical_operator(&self) -> Option<MathematicalOperator> {
        match self {
            Self::PLUS => Some(MathematicalOperator::ADD),
            Self::DASH => Some(MathematicalOperator::SUBTRACT),
            Self::ASTERISK => Some(MathematicalOperator::MULTIPLY),//be careful this isn't a pointer
            Self::FORWARDSLASH => Some(MathematicalOperator::DIVIDE),
            Self::EQUALS => Some(MathematicalOperator::ASSIGN),
            _ => None
        }
    }
}

impl MathematicalOperator {
    /**
     * the precedence of the token in expressions that is the least binding (like a comma or "=")
     */
    pub fn max_precedence() -> i32 {14}
    /**
     * the precedence of the token in expressions that is the most binding (like indexing, or pointer dereference)
     */
    pub fn min_precedence() -> i32 {1}
    
    pub fn get_precedence_level(&self) -> i32 {
        match self {
            Self::ADD => 2,
            Self::SUBTRACT => 2,
            Self::MULTIPLY => 3,
            Self::DIVIDE => 3,
            Self::ASSIGN => 14,
        }
    }

    /**
     * calculates direction(true is left to right) from a precedence level
     * note that associativity (l->r) implies searching the tokens r->l
     */
    pub fn get_associativity_direction(level: i32) -> bool {
        match level {
            1 => true,
            2 => false,
            3..=12 => true,
            13 => false,
            14 => false,
            15 => true,
            _ => panic!("unknown precedence level")
        }
    }
}