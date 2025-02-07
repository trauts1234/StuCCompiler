use crate::{ast_metadata::ASTMetadata, lexer::{token::Token, token_savepoint::TokenQueueSlice, token_walk::TokenQueue}, memory_size::MemoryLayout, number_literal::NumberLiteral, operator::Operator, stack_variables::StackVariables};
use std::fmt::Write;

#[derive(Debug)]
pub enum Expression {
    STACKVAR(MemoryLayout),//a variable that is on the stack
    NUMBER(NumberLiteral),
    BINARYEXPR(Box<Expression>, Operator, Box<Expression>)
    //ASSIGNMENT(LValue, Operator, Box<Expression>)// a = b;
}

impl Expression {
    pub fn try_consume(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<ASTMetadata<Expression>> {
        let semicolon_idx = tokens_queue.find_closure_in_slice(&previous_queue_idx, false, |x| *x == Token::PUNCTUATION(";".to_owned()))?;
        //define the slice that we are going to try and parse
        let attempt_slice = TokenQueueSlice {
            index: previous_queue_idx.index,
            max_index: semicolon_idx.index
        };

        //warning! the line of code:
        //1+1;
        //will cause a memory leak on the stack?!
        //as 1, 1 are pushed, popped, added, then 2 is pushed but nothing pops it
        //is this a problem? probably not, but it is a memory leak

        match Expression::try_consume_whole_expr(tokens_queue, &attempt_slice, local_variables) {
            Some(expr) => {
                Some(ASTMetadata{resultant_tree: expr, remaining_slice: semicolon_idx.next_clone(), extra_stack_used: MemoryLayout::new()})
            },
            None => None
        }
    }
    /**
     * tries to parse the tokens queue starting at previous_queue_idx, to find an expression
     * returns an expression(entirely consumed), else none
     */
    pub fn try_consume_whole_expr(tokens_queue: &mut TokenQueue, previous_queue_idx: &TokenQueueSlice, local_variables: &StackVariables) -> Option<Expression> {
        let mut curr_queue_idx = TokenQueueSlice::from_previous_savestate(previous_queue_idx);

        match curr_queue_idx.get_slice_size() {
            0 => panic!("not expecting this, maybe it is not an expression"),

            1 => {
                //1 token left, check if it is a number
                if let Token::NUMBER(num) = tokens_queue.peek(& curr_queue_idx)? {
                    tokens_queue.consume(&mut curr_queue_idx);
                    return Some(Expression::NUMBER(num));
                }
                //TODO match a variable
                None
            },

            _ => {
                //TODO handle brackets outside of operator

                //find highest precendence level
                let highest_precedence = tokens_queue.get_slice(&curr_queue_idx).iter()
                    .filter_map(|x| {
                        if let Token::OPERATOR(op) = x {Some(op.get_precedence_level())} else {None} //get the precedence level if it is an operator, else skip
                    })
                    .fold(std::i32::MAX, |a,b| a.min(b));//small number = great precedence

                //find which direction the operators should be considered
                let associative_direction = Operator::get_associativity_direction(highest_precedence);

                //make a closure that detects tokens that match what we want
                let operator_matching_closure = |x: &Token| {
                    match x {
                        Token::OPERATOR(op) => {op.get_precedence_level() == highest_precedence},
                        _ => false
                    }
                };

                //find first occurence of this operator, taking into account which way we have to search the array
                let first_operator_location = tokens_queue.find_closure_in_slice(&curr_queue_idx, !associative_direction, operator_matching_closure).unwrap();

                //split to before and after the operator
                let (left_part, right_part) = tokens_queue.split_to_slices(&first_operator_location, &curr_queue_idx);

                //try and parse the left and right hand sides, propogating errors
                let parsed_left = Expression::try_consume_whole_expr(tokens_queue, &left_part, local_variables)?;
                let parsed_right = Expression::try_consume_whole_expr(tokens_queue, &right_part, local_variables)?;

                let operator = match tokens_queue.peek(&first_operator_location).unwrap() {
                    Token::OPERATOR(op) => op,
                    _ => panic!("operator token is not an operator")
                };

                Some(Expression::BINARYEXPR(Box::new(parsed_left), operator, Box::new(parsed_right)))
            }
        }
    }

    /**
     * puts the result of the expression on top of the stack
     */
    pub fn generate_assembly(&self) -> String{
        let mut result = String::new();

        match self {
            Expression::NUMBER(number_literal) => {
                writeln!(result, "push {}", number_literal.nasm_format()).unwrap()
            },
            Expression::STACKVAR(stack_var) => {
                writeln!(result, "lea rax, [rbp-{}]", stack_var.size_bytes()).unwrap();//calculate the address of the variable
                writeln!(result, "push rax").unwrap();//push the address on to the stack
            },
            Expression::BINARYEXPR(lhs, operator, rhs) => {
                //push left and right hand sides on to the stack
                //warning: data type can cause headaches, especially in division + right shifts
                write!(result, "{}", lhs.generate_assembly()).unwrap();
                write!(result, "{}", rhs.generate_assembly()).unwrap();

                match operator {
                    Operator::ADD => {
                        //load values from stack
                        writeln!(result, "pop rax").unwrap();
                        writeln!(result, "pop rbx").unwrap();
                        //calculate the sum (32 bit)
                        writeln!(result, "add eax, ebx").unwrap();
                        //sign extend 32 bit result to 64 bits
                        writeln!(result, "cdq").unwrap();
                        //save the result
                        writeln!(result, "push rax").unwrap()
                        
                    }
                    Operator::MULTIPLY => {
                        //load values from stack
                        writeln!(result, "pop rax").unwrap();
                        writeln!(result, "pop rbx").unwrap();
                        //calculate the product (32 bit)
                        writeln!(result, "imul eax, ebx").unwrap();//warning: signed only
                        //sign extend 32 bit result to 64 bits
                        writeln!(result, "cdq").unwrap();
                        //save the result
                        writeln!(result, "push rax").unwrap()
                    }
                    Operator::ASSIGN => {
                        //pop the value to assign
                        writeln!(result, "pop rax").unwrap();
                        //get address to assign to
                        writeln!(result, "pop rbx").unwrap();
                        //save to memory
                        writeln!(result, "mov [rbx], rax").unwrap();
                    }
                }
            },
        }

        result
    }
}