use crate::{asm_boilerplate::{self, mov_asm}, asm_gen_data::AsmData, asm_generation::{self, asm_comment, asm_line, generate_param_reg, AssemblyOperand, LogicalRegister, RAMLocation}, classify_param::ArgType, compilation_state::functions::FunctionList, data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, expression::{self, Expression}, expression_visitors::{data_type_visitor::GetDataTypeVisitor, expr_visitor::ExprVisitor, put_scalar_in_acc::ScalarInAccVisitor, put_struct_on_stack::CopyStructVisitor}, function_declaration::FunctionDeclaration, lexer::{punctuator::Punctuator, token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, parse_data::ParseData};
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
    
    pub fn generate_assembly_scalar_return(&self, asm_data: &AsmData, stack_data: &mut MemoryLayout) -> String {
        //system V ABI
        let mut result = String::new();

        let alignment_size = MemoryLayout::from_bytes(8);//I think everything is 8 byte aligned here?

        asm_comment!(result, "calling function: {}", self.func_name);

        //attach type to each of the args
        //should this be reversed?
        let type_matched_args: Vec<_> = self.args.iter().enumerate().map(|(i, x)|{
            
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
                arg_tree: arg_data.1.clone(),
            };

            match arg_location {
                ArgType::INTEGER if acc.integer_regs_used < 6 => {
                    acc.add_integer_arg(allocated_arg, false)
                },
                ArgType::STRUCT {..} if acc.integer_regs_used <= 4 => {
                    //if there are less than 5 memory args, there is enough room for both the first and second eightbyte
                    acc.add_integer_arg(allocated_arg, true);
                }
                _ => {
                    acc.memory_args.push(allocated_arg);
                },//add if memory or if there are too many integer args, written backwards so that they are pushed forwards
            }

            acc
        });

        assert!(sorted_args.integer_args.iter().all(|x| x.param_type.decay() == x.param_type));//none can be array at this point

        //calculate stack required for the args
        let stack_required_for_memory_args: MemoryLayout = sorted_args.memory_args.iter()
            .map(|x| aligned_size(x.param_type.memory_size(asm_data), alignment_size))
            .sum();
        let stack_required_for_integer_args: MemoryLayout = sorted_args.integer_args.iter()
            .map(|x| aligned_size(x.param_type.memory_size(asm_data), alignment_size))
            .sum();
        
        //allocate stack for args passed by memory
        //TODO align to 16 bytes here
        asm_line!(result, "sub rsp, {} ; allocate memory for memory args", stack_required_for_memory_args.size_bytes());

        asm_line!(result, "{}", push_args_to_stack_backwards(
            &sorted_args.memory_args,//write memory args to stack
            asm_data,
            stack_data,//writes to [rsp+0] .. [rsp+stack_required_for_memory_args]
        ));

        //allocate stack for args to be popped to GP registers
        asm_line!(result, "sub rsp, {}", stack_required_for_integer_args.size_bytes());

        asm_line!(result, "{}", push_args_to_stack_backwards(
            &sorted_args.integer_args,//write integer args to stack
            asm_data,
            stack_data,//writes to [rsp+0] .. [rsp+stack_required_for_integer_args]
        ));
        //pop the register args to registers
        for i in 0..sorted_args.integer_regs_used {
            asm_line!(result, "pop {}", generate_param_reg(i).generate_name(alignment_size));//pop to register
        }

        asm_line!(result, "mov al, 0");//since there are no floating point args, this must be left as 0 to let varadic functions know

        asm_line!(result, "call {}", self.func_name);

        asm_line!(result, "add rsp, {} ; deallocate memory args", stack_required_for_memory_args.size_bytes());

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

#[derive(Clone)]
pub struct AllocatedArg {
    pub(crate) param_type: RecursiveDataType,//what type the arg should be cast into
    pub(crate) arg_tree: Expression
}

struct AllocatedArgs {
    integer_args: Vec<AllocatedArg>,
    integer_regs_used: usize,
    memory_args: Vec<AllocatedArg>,
}
impl AllocatedArgs {
    pub fn new() -> Self {
        AllocatedArgs { integer_args: Vec::new(), memory_args: Vec::new(), integer_regs_used:0 }
    }
    /**
     * is_double_eightbyte_struct: is the struct one that is passed by 2 registers
     * param data: the struct to be passed
     */
    pub fn add_integer_arg(&mut self, data: AllocatedArg, is_double_eightbyte_struct: bool) {
        self.integer_args.push(data);
        if !is_double_eightbyte_struct {
            self.integer_regs_used += 1;
        } else {
            self.integer_regs_used += 2;
        }
    }
}

/**
 * calculates how much extra memory is needed to make current_offset a multiple of alignment
 */
pub fn align(current_offset: MemoryLayout, alignment: MemoryLayout) -> MemoryLayout {
    let bytes_past_last_boundary = current_offset.size_bytes() % alignment.size_bytes();

    MemoryLayout::from_bytes (
        (alignment.size_bytes() - bytes_past_last_boundary) % alignment.size_bytes()
    )
}

/**
 * calculates the size of current_offset when rounded up to the alignment boundary
 * return value >= current_offset
 */
pub fn aligned_size(current_offset: MemoryLayout, alignment: MemoryLayout) -> MemoryLayout {
    current_offset + align(current_offset, alignment)
}

/**
 * pushes the args specified, aligning all to to 64 bit
 * assumes stack alignment at point where assembly is injected
 * stack_data is used for scratch space only
 * extra_stack_for_params: is the total size of extra data that will be left on the stack - this is used as a positive RSP offset to write params to
 */
fn push_args_to_stack_backwards(args: &[AllocatedArg], asm_data: &AsmData, stack_data: &mut MemoryLayout) -> String{
    let mut result = String::new();
    let alignment_size = MemoryLayout::from_bytes(8);//I think everything is 8 byte aligned here?
    let mut current_sp_offset = MemoryLayout::new();//how far from rsp is the next param

    for arg in args.iter().rev() {

        //push arg to stack
        let arg_type = arg.arg_tree.accept(&mut GetDataTypeVisitor{asm_data});

        //this code is messy:
        match (&arg_type.decay(), &arg.param_type) {
            (RecursiveDataType::RAW(BaseType::STRUCT(_)), _) => {
                asm_comment!(result, "putting struct arg on stack");

                let struct_stack_required = aligned_size(arg.param_type.memory_size(asm_data), alignment_size);

                //push struct on stack, without allocating since other variables may end up on top of stack_data
                //asm_line!(result, "sub rsp, {} ; allocate for struct param", struct_stack_required.size_bytes());
                asm_line!(result, "{}", arg.arg_tree.accept(&mut CopyStructVisitor{asm_data,stack_data, resultant_location: RAMLocation::AddToSP(current_sp_offset) }));

                current_sp_offset += struct_stack_required;//go towards sp for next param

            },
            (original_type, casted_type) => {
                asm_comment!(result, "putting arg on stack");
                assert!(original_type.memory_size(asm_data).size_bits() <= 64);

                asm_line!(result, "{}", arg.arg_tree.accept(&mut ScalarInAccVisitor{asm_data, stack_data}));//put value in acc, using standard stack to calculate it

                if casted_type != &RecursiveDataType::RAW(BaseType::VaArg) {
                    asm_line!(result, "{}", asm_boilerplate::cast_from_acc(original_type, casted_type, asm_data));//cast value if not varadic
                }

                asm_line!(result, "{}", mov_asm(alignment_size, &RAMLocation::AddToSP(current_sp_offset), &LogicalRegister::ACC));

                current_sp_offset += alignment_size;//go towards sp for next param
            }
        }
    }

    result
}