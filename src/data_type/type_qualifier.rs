use std::fmt::Display;



#[derive(Debug, Clone, PartialEq)]
pub enum TypeQualifier {
    Const,
    Volatile
}

impl Display for TypeQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            TypeQualifier::Const => "const",
            TypeQualifier::Volatile => "volatile",
        })
    }
}