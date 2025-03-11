use crate::{compilation_state::stack_variables::StackVariables, enum_definition::EnumList};

#[derive(Clone)]
pub struct ScopeData {
    pub(crate) stack_vars: StackVariables,
    pub(crate) enums: EnumList
}

impl ScopeData {
    pub fn make_empty() -> ScopeData {
        ScopeData { stack_vars: StackVariables::new(), enums: EnumList::new() }
    }
}