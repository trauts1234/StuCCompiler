use crate::{asm_boilerplate, asm_gen_data::AsmData, asm_generation::{self, asm_comment, asm_line, LogicalRegister}, classify_param::ArgType, compilation_state::functions::FunctionList, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, put_struct_on_stack::PutStructOnStack}, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData};
use std::fmt::Write;

#[derive(Clone)]
pub struct FunctionCall {
    func_name: String,//maybe an enum, for function pointers
    args: Vec<Expression>,

    decl: FunctionDeclaration
}

impl FunctionCall {
    pub fn accept<V: ExprVisitor>(&self, visitor: &mut V) -> V::Output {
        visitor.visit_func_call(self)
    }
    
    pub fn generate_assembly(&self, asm_data: &AsmData) -> String {
        //system V ABI
        let mut result = String::new();

        asm_comment!(result, "calling function: {}", self.func_name);

        //attach type to each of the args
        //should this be reversed?
        let type_matched_args: Vec<_> = self.args.iter().rev().enumerate().map(|(i, x)|{
            
            let param_type = if i >= self.decl.params.len() {
                assert!(*self.decl.params.last().unwrap().get_type() == RecursiveDataType::new(BaseType::VaArg));//more args than params, so must be varadic
                x.accept(&mut GetDataTypeVisitor{asm_data}).decay()//type is that of the arg, remembering to decay
            } else {
                self.decl.params[i].get_type().clone()//arg gets cast to param type
            };

            (param_type, x)
        }).collect();

        let sorted_args = type_matched_args.iter().fold(AllocatedArgs::new(), |mut acc, arg_data| {
            //the param type, or if it is a varadic arg, the arg type
            let type_of_curr_arg = arg_data.0.replace_va_arg(arg_data.1.accept(&mut GetDataTypeVisitor{asm_data}).decay());

            let arg_location = ArgType::param_from_type(
                &type_of_curr_arg,
                asm_data
            );

            let allocated_arg = AllocatedArg{
                param_type: type_of_curr_arg,
                arg_tree: arg_data.1,
            };

            match arg_location {
                ArgType::INTEGER if acc.integer_regs_used < 6 => acc.add_integer_arg(allocated_arg, false),
                ArgType::STRUCT {..} if acc.integer_regs_used < 5 => {
                    //if there are less than 5 memory args, there is enough room for both the first and second eightbyte
                    acc.add_integer_arg(allocated_arg, true);
                }
                _ => acc.memory_args.push(allocated_arg),//add if memory or if there are too many integer args
            }

            acc
        });

        assert!(sorted_args.integer_args.iter().all(|x| x.param_type.decay() == x.param_type));//none can be array at this point

        //TODO what if something was pushed as part of a binary expression before this was called?
        //assert!(asm_data.get_stack_height().size_bytes() % 16 == 0);//aligned for function call? perhaps I could add this to the params later

        let (memory_args_asm, memory_args_stack_usage) = push_args_to_stack(&sorted_args.memory_args, asm_data);
        
        let (integer_args_asm, integer_args_stack_usage) = push_args_to_stack(&sorted_args.integer_args, asm_data);

        let extra_stack_to_align_memory_args = align(memory_args_stack_usage, MemoryLayout::from_bytes(16));//align stack so that call happens on a 16 byte boundary
        asm_line!(result, "sub rsp, {}", extra_stack_to_align_memory_args.size_bytes());
        asm_line!(result, "{}", memory_args_asm);//write memory args first, as integer args eventually get popped

        asm_line!(result, "{}", integer_args_asm);//write integer args
        assert!(integer_args_stack_usage.size_bytes() % 8 == 0);//cannot have half a register's worth of bytes, since everything is padded to 64 bits
        let registers_required = integer_args_stack_usage.size_bits()/64;//1 register for each 64 bits
        for register_number in 0..registers_required {
            asm_line!(result, "{}", asm_boilerplate::pop_reg(&MemoryLayout::from_bytes(8), &asm_generation::generate_param_reg(register_number)));//pop aligned data to registers
        }

        asm_line!(result, "mov al, 0");//since there are no floating point args, this must be left as 0 to let varadic functions know

        asm_line!(result, "call {}", self.func_name);

        let pop_size = memory_args_stack_usage + extra_stack_to_align_memory_args;
        asm_line!(result, "add rsp, {} ; pop args passed via stack", pop_size.size_bytes());

        result
    }

    pub fn get_callee_decl(&self) -> &FunctionDeclaration {
        &self.decl
    }
    
    pub fn try_consume_whole_expr(tokens_queue: &TokenQueue, curr_queue_idx: &TokenQueueSlice, accessible_funcs: &FunctionList, scope_data: &ParseData) -> Option<FunctionCall> {
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

        let args_slices = tokens_queue.split_outside_parentheses(&all_args_slice, |x| *x == Token::PUNCTUATOR(Punctuator::COMMA));

        let mut args = Vec::new();

        if all_args_slice.get_slice_size() > 0 {//ensure args have actually been passed
            for arg_slice in args_slices {
                args.push(expression::try_consume_whole_expr(tokens_queue, &arg_slice, accessible_funcs, scope_data)?);
            }
        }

        let func_slice = TokenQueueSlice {//array or pointer etc.
            index: curr_queue_idx.index,
            max_index: curly_open_idx
        };

        if let Token::IDENTIFIER(func_name) = tokens_queue.peek(&func_slice, &scope_data)? {
            let func_decl = scope_data.get_function_declaration(&func_name).expect("found function call but no corresponding function declaration");
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

pub struct AllocatedArg<'a> {
    pub(crate) param_type: RecursiveDataType,//what type the arg should be cast into
    pub(crate) arg_tree: &'a Expression
}
struct AllocatedArgs<'a> {
    integer_args: Vec<AllocatedArg<'a>>,
    integer_regs_used: i32,
    memory_args: Vec<AllocatedArg<'a>>,
}
impl<'a> AllocatedArgs<'a> {
    pub fn new() -> Self {
        AllocatedArgs { integer_args: Vec::new(), memory_args: Vec::new(), integer_regs_used:0 }
    }
    /**
     * is_double_eightbyte_struct: is the struct one that is passed by 2 registers
     * param data: the struct to be passed
     */
    pub fn add_integer_arg(&mut self, data: AllocatedArg<'a>, is_double_eightbyte_struct: bool) {
        self.integer_args.push(data);
        if !is_double_eightbyte_struct {
            self.integer_regs_used += 1;
        } else {
            self.integer_regs_used += 2;
        }
    }
}

fn align(current_offset: MemoryLayout, alignment: MemoryLayout) -> MemoryLayout {
    let bytes_past_last_boundary = current_offset.size_bytes() % alignment.size_bytes();

    MemoryLayout::from_bytes (
        (alignment.size_bytes() - bytes_past_last_boundary) % alignment.size_bytes()
    )
}

/**
 * pushes the args specified, aligning all to to 64 bit
 * returns (assembly required, stack used to do it)
 * assumes stack alignment at point where assembly is injected
 */
fn push_args_to_stack(args: &[AllocatedArg], asm_data: &AsmData) -> (String, MemoryLayout) {
    let mut stack_taken_by_args = MemoryLayout::new();
    let mut result = String::new();

    for arg in args {
        let alignment_size = MemoryLayout::from_bytes(8);//I think everything is 8 byte aligned here?

        //increase alignment to an 8 byte boundary
        let extra_padding = align(stack_taken_by_args, alignment_size);

        stack_taken_by_args += extra_padding;//increase offset to reach optimal alignment
        asm_line!(result, "sub rsp, {} ; align for next arg", extra_padding.size_bytes());
        assert!(stack_taken_by_args.size_bytes() % 8 == 0);//ensure stack is aligned

        //push arg to stack
        let arg_type = arg.arg_tree.accept(&mut GetDataTypeVisitor{asm_data});

        //this code is messy:
        match (&arg_type.decay(), &arg.param_type) {
            (RecursiveDataType::RAW(BaseType::STRUCT(_)), _) => {
                //struct param passed via memory
                asm_comment!(result, "putting struct arg on stack");
                asm_line!(result, "{}", arg.arg_tree.accept(&mut PutStructOnStack{asm_data}));

                stack_taken_by_args += arg.param_type.memory_size(asm_data)

            },
            (original_type, RecursiveDataType::RAW(BaseType::VaArg)) => {
                asm_comment!(result, "putting varadic arg on stack");

                println!("{}", original_type.memory_size(asm_data).size_bits());
                assert!(original_type.memory_size(asm_data).size_bits() <= 64);

                asm_line!(result, "{}", arg.arg_tree.accept(&mut ScalarInAccVisitor{asm_data}));//put value in acc
                //no casting since it is a varadic arg

                asm_line!(result, "{}", asm_boilerplate::push_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//push onto stack, padding to 8 bytes as it is a primative

                stack_taken_by_args += MemoryLayout::from_bytes(8);
            },
            (original_type, casted_type) => {
                //primative type
                asm_comment!(result, "putting primative arg on stack");

                println!("{}:{}", original_type.memory_size(asm_data).size_bits(), casted_type.memory_size(asm_data).size_bits());
                assert!(original_type.memory_size(asm_data).size_bits() <= 64);
                assert!(casted_type.memory_size(asm_data).size_bits() <= 64);

                asm_line!(result, "{}", arg.arg_tree.accept(&mut ScalarInAccVisitor{asm_data}));//put value in acc
                asm_line!(result, "{}", asm_boilerplate::cast_from_acc(original_type, &casted_type, asm_data));//cast value

                asm_line!(result, "{}", asm_boilerplate::push_reg(&MemoryLayout::from_bytes(8), &LogicalRegister::ACC));//push onto stack, padding to 8 bytes as it is a primative

                stack_taken_by_args += MemoryLayout::from_bytes(8);
            }
        }
    }

    (result, stack_taken_by_args)
}