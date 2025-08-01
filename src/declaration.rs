use std::fmt::Display;

use colored::Colorize;

use crate::{data_type::recursive_data_type::DataType, expression_visitors::expr_visitor::ExprVisitor};

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

impl Display for Declaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.data_type)
    }
}

impl Display for MinimalDataVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name.blue())
    }
}