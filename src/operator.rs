
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    ADD,
    MULTIPLY,
}

impl Operator {
    pub fn try_new(to_token: &str) -> Option<Operator> {
        match to_token {
            "+" => Some(Self::ADD),
            "*" => Some(Self::MULTIPLY),
            _ => None
        }
    }
    pub fn get_precedence_level(&self) -> i32 {
        match self {
            Self::ADD => 2,
            Self::MULTIPLY => 3
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
            3 => true,
            _ => panic!("unknown precedence level")
        }
    }
}