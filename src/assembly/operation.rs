use std::fmt::Display;
use crate::{args_handling::location_allocation::{AllocatedLocation, EightByteLocation, ReturnLocation}, assembly::{assembly_text::RawAssembly, comparison::AsmComparison, operand::{register::{GPRegister, MMRegister}, Storage}}, data_type::{base_type::{BaseType, FloatType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::IRDisplay};
use memory_size::MemorySize;
use stack_management::{baked_stack_frame::BakedSimpleStackFrame, stack_item::StackItemKey};

#[derive(Clone)]
pub enum AsmOperation {
    /// Moves `size` bytes from the pointers
    MOV {from: Storage, to: Storage, size: MemorySize},

    /// Finds the address of `from` and puts a pointer to it in the eightbyte `to`
    LEA {from: Storage, to: Storage},

    /// Compare `lhs` and `rhs` using the appropriate comparison
    CMP {lhs: Storage, rhs: Storage, data_type: ScalarType},

    /// based on the comparison, sets `storage` to 1 or 0
    SETCC {to: Storage, data_type: IntegerType, comparison: AsmComparison},

    ///based on the comparison, conditionally jump to the label
    JMPCC {label: Label, comparison: AsmComparison},

    //casts and moves from -> to
    CAST {from: Storage, from_type: DataType, to: Storage, to_type: DataType},

    /// Sums lhs and rhs, storing the result in `to`
    ADD {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    ///subtracts rhs from lhs, storing the result in `to`
    SUB {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    ///multiplies lhs by rhs and stores the result in `to`
    MUL {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    /// Divides lhs by rhs and stores the result in `to`
    DIV {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},

    /// finds lhs % rhs
    MOD {lhs: Storage, rhs: Storage, to: Storage, data_type: IntegerType},

    ///shifts `from` logically left, storing the result in `to`
    SHL {from: Storage, from_type: IntegerType, amount: Storage, to: Storage},
    ///shifts `from` logically left by (u8)`amount`, storing the result in `to` (arithmetic or logical based on the signedness of `from_type`)
    SHR {from: Storage, from_type: IntegerType, amount: Storage, to: Storage},

    ///negates `from`, storing results in `to` and taking into account data type
    NEG {from: Storage, to: Storage, data_type: ScalarType},
    ///performs bitwise not to `size` bytes from `from`, storing results in `to`
    BitwiseNot {from: Storage, to: Storage, size: MemorySize},

    /// applies `operation` to `size` bytes
    BitwiseOp {lhs: Storage, rhs: Storage, to: Storage, size: MemorySize, operation: LogicalOperation},

    /// Generates an assembly label 
    Label(Label),
    /// also allocates variables on the stack
    CreateStackFrame,
    /// also (implicitly) deallocates variables on the stack
    DestroyStackFrame,
    /// Pops the return address from the stack using the `ret` instruction
    /// 
    /// Puts the return value (or hidden pointer) in correct registers
    Return {return_data: Option<(CalleeReturnData, Storage)>},
    ///calls a subroutine (you must handle the parameters though)
    /// 
    /// - Sets up registers and the stack correctly
    /// - 
    /// 
    /// TODO should this call a Storage ?
    CALL {label: String, params: Vec<CallerParamData>, return_data: Option<CallerReturnData>},

    ReadParams {regs: Vec<ReadParamFromReg>, mem: Vec<ReadParamFromMem>},

    ///not even a nop, just a blank line of assembly
    BLANK,
}

#[derive(Clone)]
pub struct ReadParamFromReg {
    /// Where the param is coming from
    pub eightbyte_locations: Vec<EightByteLocation>,
    /// How big the param is
    pub param_size: MemorySize,
    /// Where the param should be dumped into
    pub param_destination: StackItemKey
}

#[derive(Clone)]
pub struct ReadParamFromMem {
    /// How big the param is
    pub param_size: MemorySize,
    /// Where the param should be dumped into
    pub param_destination: StackItemKey
}

#[derive(Clone)]
pub struct CallerParamData {
    /// Where the arg value is
    pub data: Storage,
    /// size of the param
    pub data_size: MemorySize,
    /// where the data should be put
    pub location: AllocatedLocation,
}

#[derive(Clone)]
pub struct CallerReturnData {
    /// Where the return value will come from
    pub return_location_info: ReturnLocation,
    /// hidden pointer or not, allocate memory for the return value to go in
    pub return_location: StackItemKey,
    /// Must be aligned up to 8 bytes, so that whole 8 byte registers can be moved
    /// 
    /// TODO can this requirement be relaxed with some bit shifting magic
    pub return_location_size: MemorySize,
}

#[derive(Clone)]
pub enum CalleeReturnData {
    InRegs(Vec<EightByteLocation>),
    InMemory{hidden_pointer_location: Storage}
}

#[derive(Clone)]
pub struct MemoryArg {
    /// What data to put as a memory arg
    pub value: Storage,
    /// What to add to rsp to find the destination of the arg
    pub sp_offset: MemorySize,
    /// How many bytes to copy
    pub size: MemorySize
}

#[derive(Clone)]
pub enum LogicalOperation {
    AND,
    OR,
    XOR,
}

#[derive(Clone)]
pub enum Label {
    /// A global label
    Global(String),
    /// A local label (attaches to the previous global label)
    Local(String),
}
impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Label::Global(label) => write!(f, "{}", label),
            Label::Local(label) => write!(f, ".{}", label),
        }
    }
}

impl AsmOperation {
    /**
     * converts myself into a line of assembly, with no newline
     */
    pub fn to_text(&self, stack: &BakedSimpleStackFrame) -> RawAssembly {
        let result = RawAssembly::default();
        match self {
            AsmOperation::MOV { to, from, size } => {
                result.add_comment(format!("moving {} bytes", size.size_bytes()));
                //loop through each eightbyte(or smaller)
                for i in (0..size.size_bytes()).step_by(8) {
                    let offset = MemorySize::from_bytes(i);//find the offset into `from` and `to` that I am copying
                    let remaining_bytes = *size - offset;//find the number of bytes left to copy
                    let best_reg_size = best_reg_size(remaining_bytes);//find the biggest register size to move the next part of the data
                    let cx_name = GPRegister::_CX.generate_name(best_reg_size);//to store the bytes temporarily

                    //point to the start of the source
                    result.add(put_pointer_in_rax(from, stack));
                    //put the next bytes from the correct part of the source in RCX
                    result.add(format!("mov {}, [rax+{}]", cx_name, offset.size_bytes()));
                    //point to the start of the destination
                    result.add(put_pointer_in_rax(to, stack));
                    //store the next few bytes from RCX
                    result.add(format!("mov [rax+{}], {}", offset.size_bytes(), cx_name));
                }
            },
            
            AsmOperation::LEA { from, to } => {
                //generate address, and put in rcx
                result.add(put_pointer_in_rax(from, stack));
                result.add(format!("mov rcx, rax"));
                //get destination, and store result
                result.add(put_pointer_in_rax(to, stack));
                result.add(format!("mov [rax], rcx"));
            },
            AsmOperation::CMP { rhs, data_type, lhs } => {
                match data_type {
                    ScalarType::Float(float_type) => {
                        result.add(put_rhs_xmm0_lhs_xmm1(lhs, rhs, float_type, stack));
                        //compare
                        result.add(match float_type {
                            FloatType::F32 => "ucomiss xmm1, xmm0",
                            FloatType::F64 => "ucomisd xmm1, xmm0",
                        }.to_string());
                    },
                    ScalarType::Integer(integer_type) => {
                        result.add(put_rhs_ax_lhs_cx(lhs, rhs, integer_type, stack));
                        //compare
                        result.add(format!("cmp rax, rcx"));
                    },
                }
            },
            AsmOperation::SETCC { comparison, to, data_type } => {
                let reg_name = GPRegister::acc().generate_name(MemorySize::from_bytes(1));//setting 1 byte boolean

                let comparison_instr = match comparison {
                    AsmComparison::NE => "setne",
                    AsmComparison::EQ => "sete",
                    AsmComparison::ALWAYS => panic!(),//return format!("mov al, 1"),//for set always, there is no command, so quickly return a mov command
                    AsmComparison::LE {signed} => if *signed {"setle"} else {"setbe"},
                    AsmComparison::GE {signed} => if *signed {"setge"} else {"setae"},
                    AsmComparison::L {signed} => if *signed {"setl"} else {"setb"},
                    AsmComparison::G {signed} => if *signed {"setg"} else {"seta"},
                };

                result.add(format!("{} al", comparison_instr));
            },
            AsmOperation::JMPCC { label, comparison } => {
                let comparison_instr = match comparison {
                    AsmComparison::NE => "jne",
                    AsmComparison::EQ => "je",
                    AsmComparison::ALWAYS => "jmp",
                    AsmComparison::LE {signed}  => if *signed {"jle"} else {"jbe"},
                    AsmComparison::GE {signed}  => if *signed {"jge"} else {"jae"},
                    AsmComparison::L {signed}  => if *signed {"jl"} else {"jb"},
                    AsmComparison::G {signed}  => if *signed {"jg"} else {"ja"},
                };
                
                result.add(format!("{} {}", comparison_instr, label));
            },
            AsmOperation::ADD { data_type, lhs, rhs, to } => {
                match data_type {
                    ScalarType::Float(float_type) => todo!(),
                    ScalarType::Integer(integer_type) => {
                        let truncated_rcx = GPRegister::_CX.generate_name(integer_type.memory_size());
                        result.add(put_rhs_ax_lhs_cx(lhs, rhs, integer_type, stack));
                        //sum and put the result in rcx
                        result.add(format!("add rcx, rax"));
                        //point to the destination
                        result.add(put_pointer_in_rax(to, stack));
                        //truncate and store
                        result.add(format!("mov [rax], {}", truncated_rcx));
                    },
                }
            },
            AsmOperation::SUB { data_type } => instruction_sub(decrement, data_type, stack),
            AsmOperation::NEG { data_type } => instruction_neg(data_type),
            AsmOperation::CreateStackFrame => format!("push rbp\nmov rbp, rsp\nsub rsp, {}", stack.stack_size().size_bytes()),
            AsmOperation::DestroyStackFrame => "mov rsp, rbp\npop rbp".to_string(),
            AsmOperation::Return => "ret".to_string(),
            AsmOperation::AllocateStack(size) => format!("sub rsp, {}", size.size_bytes()),
            AsmOperation::DeallocateStack(size) => format!("add rsp, {}", size.size_bytes()),
            AsmOperation::Label(label) => format!("{}:", label),
            AsmOperation::BLANK => String::new(),
            AsmOperation::MUL { multiplier, data_type } => instruction_mul(multiplier, data_type, stack),
            AsmOperation::DIV { divisor, data_type } => instruction_div(divisor, data_type, stack),
            AsmOperation::BitwiseOp { secondary, operation} => instruction_bitwise(secondary, operation, stack),
            AsmOperation::CALL { label } => format!("call {}", label),
            AsmOperation::SHL { amount, base_type } => instruction_shiftleft(amount, base_type, stack),
            AsmOperation::SHR { amount, base_type } => instruction_shiftright(amount, base_type, stack),
            AsmOperation::BitwiseNot => format!("not {}", GPRegister::acc().generate_name(MemorySize::from_bits(64))),//just NOT the whole reg

            AsmOperation::CAST { from_type, to_type, from, to } => {
                
            }
        }

        result
    }
}

/// Says it on the tin
/// 
/// ### Clobbers
/// - RAX
/// - RCX
fn put_rhs_ax_lhs_cx(lhs:&Storage, rhs:&Storage, integer_type: &IntegerType, stack: &BakedSimpleStackFrame) -> String {
    let mut result = String::new();
    //put lhs in rcx
    result += &put_value_in_rax(lhs, integer_type, stack);
    result += &format!("mov rcx, rax");
    //put rhs in rax
    result += &put_value_in_rax(rhs, integer_type, stack);

    result
}

/// Says it on the tin
/// 
/// ### Clobbers
/// - RAX
/// - XMM0
/// - XMM1
fn put_rhs_xmm0_lhs_xmm1(lhs:&Storage, rhs:&Storage, float_type: &FloatType, stack: &BakedSimpleStackFrame) -> String {
    let mut result = String::new();
    //put lhs in XMM1
    result += &put_value_in_xmm0(lhs, float_type, stack);
    result += &format!("movq xmm1, xmm0");
    //put rhs in XMM0
    result += &put_value_in_xmm0(rhs, float_type, stack);

    result
}

/// Puts a pointer to the data in `storage` in rax
/// 
/// ### Clobbers
/// - RAX
/// ### Panics
/// if storage is a constant value, as this doesn't have an address
fn put_pointer_in_rax(storage: &Storage, stack: &BakedSimpleStackFrame) -> String {
    match storage {
        Storage::Stack(stack_item_key) => 
            format!("lea rax, [rbp-{}]", stack.get(stack_item_key).offset_from_bp.size_bytes()),
        Storage::StackWithOffset { stack: stack_item_key, offset } => 
            format!("lea rax, [rbp-{}]", (stack.get(stack_item_key).offset_from_bp + *offset).size_bytes()),//+ offset right?
        Storage::Constant(_) => panic!(),
        Storage::IndirectAddress(stack_item_key) => 
            format!("mov rax, [rbp-{}]", stack.get(stack_item_key).offset_from_bp.size_bytes()),
    }
}

/// Puts `storage` in rax, sign or zero extending to 64 bit
/// 
/// ### Clobbers
/// - RAX
fn put_value_in_rax(storage: &Storage, data_type: &IntegerType, stack: &BakedSimpleStackFrame) -> String {
    let register = GPRegister::_AX.generate_name(data_type.memory_size());
    let mut result = match storage {
        Storage::Stack(stack_item_key) => 
            format!("mov {}, [rbp-{}]", register, stack.get(stack_item_key).offset_from_bp.size_bytes()),
        Storage::StackWithOffset { stack: stack_item_key, offset } => 
            format!("mov {}, [rbp-{}]", register, (stack.get(stack_item_key).offset_from_bp + *offset).size_bytes()),//+ offset right?
        Storage::Constant(number_literal) => 
            format!("mov {}, {}", register, number_literal.generate_nasm_literal()),
        Storage::IndirectAddress(stack_item_key) => 
            format!("mov rax, [rbp-{}]\nmov {}, [rax]", stack.get(stack_item_key).offset_from_bp.size_bytes(), register),
    };

    //sign extend
    result += 
        if data_type.is_unsigned() {
            zero_extend(data_type.memory_size())
        } else {
            sign_extend(data_type.memory_size())
        };

    result
}

/// Puts `storage` in XMM0
/// 
/// ### Clobbers
/// - RAX
/// - XMM0
fn put_value_in_xmm0(storage: &Storage, data_type: &FloatType, stack: &BakedSimpleStackFrame) -> String {
    let mov_from_mem = match data_type {
        FloatType::F32 => "movss",
        FloatType::F64 => "movsd",
    };
    let mov_from_reg = match data_type {
        FloatType::F32 => "movd",
        FloatType::F64 => "movq",
    };

    match storage {
        Storage::Stack(stack_item_key) => 
            format!("{} xmm0, [rbp-{}]", mov_from_mem, stack.get(stack_item_key).offset_from_bp.size_bytes()),
        Storage::StackWithOffset { stack: stack_item_key, offset } => 
            format!("{} xmm0, [rbp-{}]", mov_from_mem, (stack.get(stack_item_key).offset_from_bp + *offset).size_bytes()),//+ offset right?
        Storage::Constant(number_literal) => 
            format!("mov rax, {}\n{} xmm0, rax", number_literal.generate_nasm_literal(), mov_from_reg),//pass the raw bitpattern via rax
        Storage::IndirectAddress(stack_item_key) => 
            format!("mov rax, [rbp-{}]\n{} xmm0, [rax]", stack.get(stack_item_key).offset_from_bp.size_bytes(), mov_from_mem),
    }
}

fn instruction_cast(from_type: &ScalarType, to_type: &ScalarType) -> String {
    match (from_type, to_type) {
        (ScalarType::Integer(lhs), ScalarType::Integer(IntegerType::_BOOL)) => {
            //boolean, so I need to cmp 0
            format!("cmp {}, 0\nsetne al", GPRegister::acc().generate_name(lhs.memory_size()))
        }
        (ScalarType::Integer(lhs), ScalarType::Integer(_)) => {
            let lhs_original_size = lhs.memory_size();
            if lhs.is_unsigned() {
                zero_extend(&lhs_original_size).to_string()//extend to 64 bits, as truncation is implicit
            } else {
                sign_extend(&lhs_original_size)// ''
            }
        },

        //float to integer conversions
        (ScalarType::Float(lhs), ScalarType::Integer(IntegerType::_BOOL)) => todo!(),
        (ScalarType::Float(FloatType::F32), ScalarType::Integer(y)) => format!("{}\ncvtss2si rax, xmm0", acc_to_xmm()),
        (ScalarType::Float(FloatType::F64), ScalarType::Integer(y)) => format!("{}\ncvtsd2si rax, xmm0", acc_to_xmm()),

        (ScalarType::Integer(IntegerType::U64), ScalarType::Float(_)) => todo!("this is difficult :("),

        //definitely not u64, so cast to i64 as that should fit anything, then cast to float
        //don't forget to put the results back in acc
        (ScalarType::Integer(_), ScalarType::Float(FloatType::F32)) => format!("{}\ncvtsi2ss xmm0, rax\n{}", instruction_cast(from_type, &ScalarType::Integer(IntegerType::I64)), xmm_to_acc()),
        (ScalarType::Integer(_), ScalarType::Float(FloatType::F64)) => format!("{}\ncvtsi2sd xmm0, rax\n{}", instruction_cast(from_type, &ScalarType::Integer(IntegerType::I64)), xmm_to_acc()),

        //float-float casts
        (ScalarType::Float(lhs), ScalarType::Float(rhs)) => match (lhs, rhs) {
            (FloatType::F32, FloatType::F32) => String::new(),
            (FloatType::F64, FloatType::F64) => String::new(),
            //don't forget to grab from rax
            (FloatType::F32, FloatType::F64) => format!("{}\ncvtss2sd xmm0, xmm0\n{}", acc_to_xmm(), xmm_to_acc()),
            (FloatType::F64, FloatType::F32) => format!("{}\ncvtsd2ss xmm0, xmm0\n{}", acc_to_xmm(), xmm_to_acc()),
        }
    }
}

/// Writes instructions to sign extend the accumulator
fn sign_extend(original: MemorySize) -> &'static str {
    match original.size_bytes() {
        1 => "cbw\ncwde\ncdqe",
        2 => "cwde\ncqde",
        4 => "cdqe",
        8 => "",
        _ => panic!("tried to sign extend unknown size")
    }
}

/// Writes instructions to zero extend the accumulator
fn zero_extend(original: MemorySize) -> &'static str {
    match original.size_bytes() {
        1 => "movzx rax, al",
        2 => "movzx rax, ax",
        4 => "",// Writing to EAX automatically zeroes RAX's upper half.
        8 => "",//already right size
        _ => panic!("tried to zero extend unknown size")
    }
}

fn instruction_add(increment: &Operand, data_type: &ScalarType, stack: &BakedSimpleStackFrame) -> String {
    match data_type {
        //addition is same for signed and unsigned
        ScalarType::Integer(base) => format!("add {}, {}", GPRegister::acc().generate_name(base.memory_size()), increment.generate_name(base.memory_size(), stack)),
        ScalarType::Float(FloatType::F32) => format!("{}\n{}\naddss xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), increment, MemorySize::from_bytes(4), stack), xmm_to_acc()),
        ScalarType::Float(FloatType::F64) => format!("{}\n{}\naddsd xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), increment, MemorySize::from_bytes(8), stack), xmm_to_acc()),
    }
}
fn instruction_sub(decrement: &Operand, data_type: &ScalarType, stack: &BakedSimpleStackFrame) -> String {
    match data_type {
        //subtraction is same for signed and unsigned
        ScalarType::Integer(base) => format!("sub {}, {}", GPRegister::acc().generate_name(base.memory_size()), decrement.generate_name(base.memory_size(), stack)),
        ScalarType::Float(FloatType::F32) => format!("{}\n{}\nsubss xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), decrement, MemorySize::from_bytes(4), stack), xmm_to_acc()),
        ScalarType::Float(FloatType::F64) => format!("{}\n{}\nsubsd xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), decrement, MemorySize::from_bytes(8), stack), xmm_to_acc()),
    }
}

fn instruction_neg(data_type: &ScalarType) -> String {
    match data_type {
        ScalarType::Float(FloatType::F32) => format!("{}\nxorps {}, [FLOAT_NEGATE]\n{}", acc_to_xmm(), MMRegister::acc().generate_name(MemorySize::from_bytes(4)), xmm_to_acc()),
        ScalarType::Float(FloatType::F64) => format!("{}\nxorps {}, [DOUBLE_NEGATE]\n{}", acc_to_xmm(), MMRegister::acc().generate_name(MemorySize::from_bytes(4)), xmm_to_acc()),
        ScalarType::Integer(integer_type) => format!("neg {}", GPRegister::acc().generate_name(integer_type.memory_size())),
    }
}

fn instruction_div(divisor: &RegOrMem, data_type: &ScalarType, stack: &BakedSimpleStackFrame) -> String {
    match data_type {
        ScalarType::Integer(IntegerType::I32) => format!("cdq\nidiv {}", divisor.generate_name(MemorySize::from_bytes(4), stack)),
        ScalarType::Integer(IntegerType::I64) => format!("cqo\nidiv {}", divisor.generate_name(MemorySize::from_bytes(8), stack)),
        ScalarType::Integer(IntegerType::U32) => format!("mov edx, 0\ndiv {}", divisor.generate_name(MemorySize::from_bytes(4), stack)),
        ScalarType::Integer(IntegerType::U64) => format!("mov rdx, 0\ndiv {}", divisor.generate_name(MemorySize::from_bytes(8), stack)),

        ScalarType::Float(FloatType::F32) => format!("{}\n{}\ndivss xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), &divisor.clone().into(), MemorySize::from_bytes(4), stack), xmm_to_acc()),
        ScalarType::Float(FloatType::F64) => format!("{}\n{}\ndivsd xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), &divisor.clone().into(), MemorySize::from_bytes(8), stack), xmm_to_acc()),
        x => panic!("cannot divide by this type: {}", x)
    }
}

fn instruction_mul(multiplier: &RegOrMem, data_type: &ScalarType, stack: &BakedSimpleStackFrame) -> String {
    //todo scalar type only
    match data_type {
        ScalarType::Integer(base) => if base.is_unsigned() {
            format!("mul {}", multiplier.generate_name(base.memory_size(), stack))
        } else {
            format!("imul {}", multiplier.generate_name(base.memory_size(), stack))
        },
        ScalarType::Float(FloatType::F32) => format!("{}\n{}\nmulss xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), &multiplier.clone().into(), MemorySize::from_bytes(4), stack), xmm_to_acc()),
        ScalarType::Float(FloatType::F64) => format!("{}\n{}\nmulsd xmm0, xmm1\n{}", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), &multiplier.clone().into(), MemorySize::from_bytes(8), stack), xmm_to_acc()),
    }
    
}

fn instruction_bitwise( secondary: &Operand, operation: &LogicalOperation, stack: &BakedSimpleStackFrame) -> String {
    let op_asm = match operation {
        LogicalOperation::AND => "and".to_string(),
        LogicalOperation::OR => "or".to_string(),
        LogicalOperation::XOR => "xor".to_string()
    };

    let (primary_name, secondary_name) = match secondary {
        Operand::GPReg(gpregister) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), gpregister.generate_name(MemorySize::from_bits(64))),//TODO ensure AX is active
        Operand::MMReg(mmregister) => panic!(),
        Operand::Mem(memory_operand) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), memory_operand.generate_name(stack)),
        Operand::Imm(immediate_value) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), immediate_value.generate_name()),
    };

    format!("{} {}, {}", op_asm, primary_name, secondary_name)
}

fn instruction_shiftleft(amount: &Operand, base_type: &BaseType, stack: &BakedSimpleStackFrame) -> String {
    let size = base_type.get_non_struct_memory_size();
    format!("shl {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1), stack))
}

fn instruction_shiftright(amount: &Operand, base_type: &BaseType, stack: &BakedSimpleStackFrame) -> String {
    let size = base_type.get_non_struct_memory_size();//TODO this should be integer only???
    match base_type {
        //signed shift needs algebraic shift right
        base if base.is_signed() => format!("sar {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1), stack)),
        //unsigned uses logical shift
        base if base.is_unsigned() => format!("shr {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1), stack)),
        _ => panic!("cannot shift this type")
    }
}

fn best_reg_size(x: MemorySize) -> MemorySize {
    assert_ne!(x, MemorySize::default());//cannot have a 0 register size

    MemorySize::from_bytes(
        1 << (64 - x.size_bytes().leading_zeros())
    )
    .min(MemorySize::from_bytes(8))//GP registers are only 8 byte max
}

impl IRDisplay for AsmOperation {
    fn display_ir(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => format!("{} = {} ({})", to, from, size),
            AsmOperation::BLANK => String::new(),
            AsmOperation::LEA { from, to } => format!("{} = &{}", to, from),
            AsmOperation::CMP { lhs, rhs, data_type } => format!("compare {}, {} ({})", lhs, rhs, data_type),
            AsmOperation::SETCC { to, data_type, comparison } => format!("set-{} {} ({})", comparison, to, data_type),
            AsmOperation::JMPCC { label, comparison } => format!("jump-{} to {}", comparison, label),
            AsmOperation::CAST { from, from_type, to, to_type } => format!("cast {} -> {} ({} = {})", from_type, to_type, to, from),
            AsmOperation::ADD { lhs, rhs, to, data_type } => format!("{} = {} + {} ({})", to, lhs, rhs, data_type),
            AsmOperation::SUB { lhs, rhs, to, data_type } => format!("{} = {} - {} ({})", to, lhs, rhs, data_type),
            AsmOperation::MUL { lhs, rhs, to, data_type } => format!("{} = {} * {} ({})", to, lhs, rhs, data_type),
            AsmOperation::DIV { lhs, rhs, to, data_type } => format!("{} = {} / {} ({})", to, lhs, rhs, data_type),
            AsmOperation::MOD { lhs, rhs, to, data_type } => format!("{} = {} % {} ({})", to, lhs, rhs, data_type),
            AsmOperation::SHL { from, from_type, amount, to } => format!("{} = {} << {} ({})", to, from, amount, from_type),
            AsmOperation::SHR { from, from_type, amount, to } => format!("{} = {} >> {} ({})", to, from, amount, from_type),
            AsmOperation::NEG { from, to, data_type } => format!("{} = -{} ({})", to, from, data_type),
            AsmOperation::BitwiseNot { from, to, size } => format!("{} = ~{} ({})", to, from, size),
            AsmOperation::BitwiseOp { lhs, rhs, to, size, operation } => format!("{} = {} {} {} ({})", to, lhs, operation, rhs, size),
            AsmOperation::Label(label) => format!("{}", label),
            AsmOperation::CreateStackFrame => format!("create stack frame and reserve stack space"),
            AsmOperation::DestroyStackFrame => format!("destroy stack frame"),
            AsmOperation::Return { return_data: None } => format!("return"),
            AsmOperation::Return { return_data: Some((return_location, storage)) } => format!("return {}", storage),
            AsmOperation::CALL { label, params, return_data } => format!("call {}", label),
            AsmOperation::ReadParams { regs, mem } => format!("load params"),
        }
    }
}

impl Display for LogicalOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            LogicalOperation::AND => "&=",
            LogicalOperation::OR => "|=",
            LogicalOperation::XOR => "^=",
        })
    }
}

impl Display for AsmComparison {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            AsmComparison::ALWAYS => "always",
            AsmComparison::NE => "ne",
            AsmComparison::EQ => "eq",
            AsmComparison::LE {..} => "le",
            AsmComparison::GE {..} => "ge",
            AsmComparison::L {..} => "l",
            AsmComparison::G {..} => "g",
        })
    }
}

