use crate::{asm_gen_data::{AsmData, GetStruct, GlobalAsmData}, assembly::{assembly::Assembly, operand::{immediate::ToImmediate, Operand}, operation::AsmOperation}, data_type::{base_type::{BaseType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::ASTDisplay, expression::expression::Expression, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, reference_assembly_visitor::ReferenceVisitor}, stack_allocation::StackAllocator};
use unwrap_let::unwrap_let;

#[derive(Clone, Debug)]
pub struct StructMemberAccess {
    struct_tree: Box<Expression>,//need a tree or something to represent what gives me back the struct
    member_name: String,
}

impl StructMemberAccess {
    pub fn new(struct_tree: Expression, member_name: String) -> StructMemberAccess {
        StructMemberAccess { struct_tree: Box::new(struct_tree), member_name }
    }

    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_struct_member_access(self)
    }

    pub fn get_base_struct_tree(&self) -> &Expression {
        &self.struct_tree
    }
    pub fn get_member_name(&self) -> &str {
        &self.member_name
    }

    pub fn get_data_type(&self, asm_data: &AsmData) -> DataType {
        let struct_tree_type = self.struct_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct

        unwrap_let!(DataType::RAW(BaseType::Struct(struct_name)) = struct_tree_type);

        let (member_decl, _) = asm_data.get_struct(&struct_name).get_member_data(&self.member_name);//get the type of the member

        member_decl.data_type.clone()
    }

    pub fn put_addr_in_acc(&self, asm_data: &AsmData, stack_data: &mut StackAllocator, global_asm_data: &mut GlobalAsmData) -> Assembly {
        let mut result = Assembly::make_empty();

        result.add_comment(format!("getting address of struct's member {}", self.member_name));
        //put tree's address in acc
        //add the member offset

        let base_struct_address_asm = self.struct_tree.accept(&mut ReferenceVisitor {asm_data, stack_data, global_asm_data});//assembly to get address of struct

        let base_struct_type = self.struct_tree.accept(&mut GetDataTypeVisitor {asm_data});//get type of the tree that returns the struct

        unwrap_let!(DataType::RAW(BaseType::Struct(struct_name)) = base_struct_type);

        let (_, struct_member_offset) = asm_data.get_struct(&struct_name).get_member_data(&self.member_name);//get offset for the specific member

        //get address of struct
        result.merge(&base_struct_address_asm);

        //go up by member offset
        result.add_instruction(AsmOperation::ADD {
            increment: Operand::Imm(struct_member_offset.as_imm()),
            data_type: ScalarType::Integer(IntegerType::U64)//pointer addition is u64 add
        });

        result
    }
}

impl ASTDisplay for StructMemberAccess {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("access struct member {}", self.member_name));
        f.indent();
        self.struct_tree.display_ast(f);
        f.dedent();
    }
}