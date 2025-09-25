use crate::{args_handling::location_allocation::generate_param_and_return_locations, asm_gen_data::AsmData, assembly::{assembly::Assembly, operand::Storage, operation::{AsmOperation, CallerParamData, CallerReturnData}}, data_type::{base_type::{BaseType, FloatType, ScalarType}, recursive_data_type::{calculate_unary_type_arithmetic, DataType}}, debugging::ASTDisplay, expression::expression::{self, promote, Expression}, expression_visitors::expr_visitor::ExprVisitor, function_declaration::FunctionDeclaration, generate_ir_traits::{GenerateIR, GetType}, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::{TokenQueue, TokenSearchType}}, parse_data::ParseData};
use memory_size::MemorySize;
use stack_management::simple_stack_frame::SimpleStackFrame;

#[derive(Clone, Debug)]
pub struct FunctionCall {
    func_name: String,//maybe an enum, for function pointers
    args: Vec<Expression>,

    decl: FunctionDeclaration
}

impl FunctionCall {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_func_call(self)
    }

    pub fn get_callee_decl(&self) -> &FunctionDeclaration {
        &self.decl
    }
    
    pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, scope_data: &mut ParseData) -> Option<FunctionCall> {
        //look for unary postfixes as association is left to right
        let last_token = tokens_queue.peek_back(&curr_queue_idx, &scope_data)?;
    
        if last_token != Token::PUNCTUATOR(Punctuator::CLOSECURLY){
            return None;
        }
    
        let curly_open_idx = tokens_queue.find_matching_open_bracket(curr_queue_idx.max_index-1);//-1 as max index is exclusive
    
        let all_args_slice = TokenQueueSlice {
            index: curly_open_idx+1,
            max_index: curr_queue_idx.max_index-1
        };

        let args_slices = tokens_queue.split_outside_parentheses(&all_args_slice, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA), &TokenSearchType::skip_all_brackets());

        let mut args = Vec::new();

        if all_args_slice.get_slice_size() > 0 {//ensure args have actually been passed
            for arg_slice in args_slices {
                args.push(expression::try_consume_whole_expr(tokens_queue, &arg_slice, scope_data)?);
            }
        }

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: curly_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice, &scope_data)? {
            //warning: label definition in a bad place will trip this up:

            //label: printf(""); will be interpreted at label()
            let func_decl = scope_data.get_function_declaration(&func_name).expect(&format!("found function call but no corresponding function declaration: {}", func_name));
            Some(FunctionCall {
                func_name, 
                args,
                decl: func_decl.clone(),
            })
        } else {
            None
        }
    }
}

impl GenerateIR for FunctionCall {
    fn generate_ir(&self, asm_data: &AsmData, stack_data: &mut SimpleStackFrame, global_asm_data: &crate::asm_gen_data::GlobalAsmData) -> (Assembly, Option<stack_management::stack_item::StackItemKey>) {
        //system V ABI
        let mut result = Assembly::make_empty();

        result.add_comment(format!("calling function: {}", self.func_name));

        let has_va_args = self.decl.params.last().is_some_and(|x| x.data_type == DataType::new(BaseType::VaArg));

        //attach type to each of the args
        let type_matched_args: Vec<_> = self.args.iter()
            .enumerate()
            .map(|(i, expr)|{
                let last_param_or_after = i >= (self.decl.params.len()-1);
                let param_type = if has_va_args && last_param_or_after{
                    //promotion of the arg is required, subject to some funny rules
                    match expr.get_type(asm_data).decay() {
                        DataType::RAW(BaseType::Scalar(ScalarType::Float(_))) => DataType::RAW(BaseType::Scalar(ScalarType::Float(FloatType::F64))),//for some reason, varadic args request promotion to f64
                        x => calculate_unary_type_arithmetic(&x)//promote the param via C99, §6.5.2.2/6
                    }
                } else {
                    self.decl.params[i].data_type.clone()//arg gets cast to param type
                };

                (param_type, expr)
            })
            .collect();

        let (return_location_type, params_locations) = generate_param_and_return_locations(type_matched_args.iter().map(|(t, _)| t), &self.decl.return_type, asm_data);
        
        let return_data = return_location_type
            .map(|return_location_info| {
                let return_location_size = self.decl.return_type.memory_size(asm_data).align_up(&MemorySize::from_bytes(8));
                let return_location = stack_data.allocate(return_location_size);
                CallerReturnData {return_location_info, return_location, return_location_size}
            });

        assert!(type_matched_args.iter().all(|x| x.0.decay() == x.0));//none can be array at this point

        let args_generated = 
            type_matched_args.into_iter()
            .zip(params_locations.into_iter())
            .map(|((dtype, expr), location)| {
                let expr_type = expr.get_type(asm_data);
                let casted_size = dtype.memory_size(asm_data);

                //generate the arg value
                let (arg_asm, value) = expr.generate_ir(asm_data, stack_data, global_asm_data);
                result.merge(&arg_asm);
                //promote the arg correctly
                let (promote_asm, promote_value) = promote(value.unwrap(), expr_type, dtype, stack_data, asm_data);
                result.add_instruction(promote_asm);

                CallerParamData {
                    data: Storage::Stack(promote_value),
                    data_size: casted_size,
                    location,
                }
            })
            .collect();

        result.add_instruction(AsmOperation::CALL {
            label: self.func_name.clone(),
            params: args_generated,
            return_data: return_data.clone(),
        });

        (result, return_data.map(|x| x.return_location))
    }
}

impl GetType for FunctionCall {
    fn get_type(&self, asm_data: &AsmData) -> DataType {
        self.get_callee_decl().return_type.clone()
    }
}

impl ASTDisplay for FunctionCall {
    fn display_ast(&self, f: &mut crate::debugging::TreeDisplayInfo) {
        f.write(&format!("call {}", self.func_name));
        f.indent();
        for i in &self.args {
            i.display_ast(f);
        }
        f.dedent();
    }
}