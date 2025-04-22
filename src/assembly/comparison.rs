#[derive(Clone)]
pub enum AsmComparison {
    ALWAYS,//always jump or set to true
    NE,//not equal
    EQ,//equal
    ///less than or equal to
    LE {signed: bool},
    ///greater than or equal to
    GE {signed: bool},
    ///less than
    L {signed: bool},
    ///greater than
    G {signed: bool},
}

pub enum ComparisonKind {
    ALWAYS,//always jump or set to true
    NE,//not equal
    EQ,//equal
    ///less than or equal to
    LE,
    ///greater than or equal to
    GE,
    ///less than
    L,
    ///greater than
    G,
}

impl AsmComparison {
    /// works out what type of comparison is represented, regardless of signedness
    pub fn kind(&self) -> ComparisonKind {
        match self {
            Self::ALWAYS => ComparisonKind::ALWAYS,
            Self::NE => ComparisonKind::NE,
            Self::EQ => ComparisonKind::EQ,
            Self::LE { signed:_ } => ComparisonKind::LE,
            Self::GE { signed:_ } => ComparisonKind::GE,
            Self::L { signed:_ } => ComparisonKind::L,
            Self::G { signed:_ } => ComparisonKind::G,
        }
    }
}

impl ComparisonKind {
    pub fn to_asm_comparison(&self, signed: bool) -> AsmComparison {
        match self {
            Self::ALWAYS => AsmComparison::ALWAYS,
            Self::NE => AsmComparison::NE,
            Self::EQ => AsmComparison::EQ,
            Self::LE => AsmComparison::LE { signed },
            Self::GE => AsmComparison::GE { signed },
            Self::L => AsmComparison::L { signed },
            Self::G => AsmComparison::G { signed },
        }
    }
}