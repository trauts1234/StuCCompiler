
#[derive(Debug, Clone, PartialEq)]
pub enum Operator {
    ADD,
}

impl Operator {
    pub fn try_new(to_token: &str) -> Option<Operator> {
        match to_token {
            "+" => Some(Self::ADD),
            _ => None
        }
    }
    pub fn get_precedence_level(&self) -> i32 {
        match self {
            Self::ADD => 2,
            _ => panic!("unknown operator precedence for this operator")
        }
    }

    /**
     * calculates direction(true is left to right) from a precedence level
     */
    pub fn get_precedence_direction(level: i32) -> bool {
        match level {
            1 => true,
            2 => false,
            _ => panic!("unknown precedence level")
        }
    }
}