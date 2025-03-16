use crate::{compilation_state::stack_variables::StackVariables, enum_definition::EnumList, function_declaration::FunctionDeclaration};

#[derive(Clone, Debug)]
pub struct ScopeData {
    pub(crate) stack_vars: StackVariables,
    pub(crate) enums: EnumList,
    function_decls: Vec<FunctionDeclaration>
}

impl ScopeData {
    pub fn make_empty() -> ScopeData {
        ScopeData { stack_vars: StackVariables::new(), enums: EnumList::new(), function_decls: Vec::new() }
    }

    pub fn func_declarations_as_slice(&self) -> &[FunctionDeclaration] {
        self.function_decls.as_slice()
    }
    
    pub fn add_declaration(&mut self, toadd: FunctionDeclaration) {
        if self.get_function_declaration(&toadd.function_name).is_some() {
            return;//already declared, skip it
        }

        self.function_decls.push(toadd);
    }

    pub fn get_function_declaration(&self, func_name: &str) -> Option<&FunctionDeclaration> {
        self.function_decls.iter()
        .find(|func| func.function_name == func_name)
    }
}