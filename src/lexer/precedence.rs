/**
     * the precedence of the token in expressions that is the least binding (like a comma or "=")
     */
    pub fn max_precedence() -> i32 {14}
    /**
     * the precedence of the token in expressions that is the most binding (like indexing, or pointer dereference)
     */
    pub fn min_precedence() -> i32 {1}

    /**
     * calculates direction(true is left to right) from a precedence level
     * note that associativity (l->r) implies searching the tokens r->l
     */
    pub fn get_associativity_direction(precedence_level: i32) -> bool {
        match precedence_level {
            1 => true,
            2 => false,
            3..=12 => true,
            13 => false,
            14 => false,
            15 => true,
            _ => panic!("unknown precedence level")
        }
    }