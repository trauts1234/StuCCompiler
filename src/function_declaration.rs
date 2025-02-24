use crate::{declaration::Declaration, type_info::DataType};

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub(crate) function_name: String,
    pub(crate) params: Vec<Declaration>,//should this be a data type?
    pub(crate) return_type: DataType,
}