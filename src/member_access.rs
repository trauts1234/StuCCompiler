use crate::{asm_gen_data::{AsmData, GetStructUnion, GlobalAsmData}, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, Operand}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}};
use memory_size::MemorySize;
use stack_management::simple_stack_frame::SimpleStackFrame;

#[derive(Clone, Debug)]
pub struct MemberAccess {
    base_tree: Box<Expression>,//need a tree or something to represent what gives me back the struct/union
    member_name: String,
}

impl MemberAccess {
    pub fn new(base_tree: Expression, member_name: String) -> MemberAccess {
        MemberAccess { base_tree: Box::new(base_tree), member_name }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_member_access(self)
    }

    pub fn get_base_tree(&self) -> &Expression {
        &self.base_tree
    }
    pub fn get_member_name(&self) -> &str {
        &self.member_name
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let base_tree_type = self.base_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct/union

        match base_tree_type {
            DataType::RAW(BaseType::Struct(struct_name)) => {
                let (member_decl, _) = asm_data.get_struct(&struct_name).get_member_data(&self.member_name);//get the type of the member

                member_decl.data_type.clone()
            }

            DataType::RAW(BaseType::Union(union_name)) => {
                asm_data.get_union(&union_name)
                .get_member_data(&self.member_name)
                .data_type.clone()
            }
            _ => panic!("this base type doesn't have members?")
        }

        
    }

    pub fn put_addr_in_acc(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("getting address of member {}", self.member_name));
        //put tree's address in acc
        //add the member offset

        let base_address_asm = self.base_tree.accept(&mut ReferenceVisitor {asm_data, stack_data, global_asm_data});//assembly to get address of struct

        let base_type = self.base_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct

        let member_offset = match base_type {
            DataType::RAW(BaseType::Struct(struct_name)) => {
                asm_data.get_struct(&struct_name).get_member_data(&self.member_name).1
            },
            DataType::RAW(BaseType::Union(_)) => {
                MemorySize::new()//everything in a union is at offset 0
            }
            _ => panic!("this type doesn't have members?")
        };

        //get address of struct
        result.merge(&base_address_asm);

        //go up by member offset
        result.add_instruction(AsmOperation::ADD {
            increment: Operand::Imm(member_offset.as_imm()),
            data_type: ScalarType::Integer(IntegerType::U64)//pointer addition is u64 add
        });

        result
    }
}

impl ASTDisplay for MemberAccess {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("access member {}", self.member_name));
        f.indent();
        self.base_tree.display_ast(f);
        f.dedent();
    }
}