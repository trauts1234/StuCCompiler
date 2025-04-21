use crate::{data_type::recursive_data_type::DataType, debugging::{ASTDisplay, DebugDisplay}, expression_visitors::expr_visitor::ExprVisitor};

#[derive(Clone, Debug)]
/**
 * stores enough data to know about a variable, using available context during assembly generation
 */
pub struct MinimalDataVariable {
    pub(crate) name: String
}

impl MinimalDataVariable {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_variable(self)
    }
}

/**
 * stores enough data to declare a variable:
 * name and data type
 */
#[derive(Debug, Clone, PartialEq)]
pub struct Declaration {
    pub(crate) data_type: DataType,
    pub(crate) name: String,
}

impl Declaration {
    pub fn get_type(&self) -> &DataType {
        //maybe unused
        &self.data_type
    }
    pub fn get_name(&self) -> &str {
        &self.name
    }

    
}

impl DebugDisplay for Declaration {
    fn display(&self) -> String {
        format!("{}: {}", self.get_name(), self.get_type().display())
    }
}

impl ASTDisplay for MinimalDataVariable {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&self.name);
    }
}