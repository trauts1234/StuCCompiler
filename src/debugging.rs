

pub trait IRDisplay {
    fn display_ir(&self) -> String;
}

pub trait ASTDisplay {
    fn display_ast(&self) -> String;
}

pub trait DebugDisplay {
    fn display(&self) -> String;
}