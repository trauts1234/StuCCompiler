
pub struct Identifier {
    name: String
}

impl Identifier{
    pub fn try_new(to_token: &str) -> Option<Identifier> {
        Some( Identifier{
            name: to_token.to_string()
        })
    }
}