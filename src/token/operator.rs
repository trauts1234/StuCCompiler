
pub enum Operator {
    PLUS,
    //todo add more
}

impl Operator {
    pub fn try_new(to_token: &str) -> Option<Operator>{
        match to_token {
            "+" => Some(Self::PLUS),
            _ => None
        }
    }
}