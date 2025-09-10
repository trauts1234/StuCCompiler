
    // fn visit_func_call(&mut self, func_call: &crate::function_call::FunctionCall) -> Self::Output {
    //     let mut result = Assembly::make_empty();

    //     if let DataType::RAW(BaseType::Struct(struct_name)) = func_call.accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}) {
    //         let struct_type = self.asm_data.get_struct(&struct_name);
    //         todo!("detect whether the struct is MEMORY or other, then allocate a hidden param or read from registers after function has been called. remember to align the stack")
    //     } else {
    //         panic!("Expected a struct type in function call");
    //     }

    //     result
    // }

    /*
     * node: this does not allocate, since the pointer points to already-allocated memory
     */
    // fn visit_unary_prefix(&mut self, expr: &UnaryPrefixExpression) -> Self::Output {
    //     let mut result = Assembly::make_empty();
    //     assert!(*expr.get_operator() == UnaryPrefixOperator::Dereference);// unary prefix can only return a struct when it is a dereference operation
        
    //     let expr_addr_asm = expr.accept(&mut ReferenceVisitor{asm_data:self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data});
    //     result.merge(&expr_addr_asm);

    //     let dereferenced_size = expr.accept(&mut GetDataTypeVisitor{asm_data:self.asm_data}).memory_size(self.asm_data);
    //     let struct_clone_asm = clone_struct_to_stack(dereferenced_size, &self.resultant_location);
    //     result.merge(&struct_clone_asm);

    //     result
    // }

    // fn visit_member_access(&mut self, member_access: &MemberAccess) -> Self::Output {
    //     //this function handles getting struct members that are also structs themselves
    //     let mut result = Assembly::make_empty();

    //     let member_name = member_access.get_member_name();
    //     unwrap_let!(DataType::RAW(BaseType::Struct(original_struct_name)) = member_access.get_base_tree().accept(&mut GetDataTypeVisitor{asm_data: self.asm_data}));
    //     let member_data = self.asm_data.get_struct(&original_struct_name).get_member_data(member_name);

    //     //generate the outer struct that I am going to be getting a member of
    //     let generate_struct_base = member_access.get_base_tree().accept(&mut CopyStructVisitor{asm_data: self.asm_data, stack_data: self.stack_data, global_asm_data: self.global_asm_data, resultant_location_ptr: self.resultant_location_ptr});
    //     result.merge(&generate_struct_base);

    //     result.add_comment(format!("increasing pointer to get index of member struct {}", member_data.0.name));

    //     //increase pointer to index of member
    //     result.add_instruction(AsmOperation::ADD {
    //         increment: Operand::Imm(member_data.1.as_imm()),
    //         data_type: ScalarType::Integer(IntegerType::U64),
    //     });

    //     result
    // }