use std::{fmt::Display, num::NonZeroU64};

use crate::{assembly::{comparison::AsmComparison, operand::{register::{GPRegister, MMRegister, Register}, Storage}}, data_type::base_type::{BaseType, FloatType, IntegerType, ScalarType}, debugging::IRDisplay};
use memory_size::MemorySize;
use stack_management::baked_stack_frame::BakedSimpleStackFrame;
use super::{operand::{Operand, PTR_SIZE}};


#[derive(Clone)]
pub enum AsmOperation {
    /// Moves `size` bytes from the pointers
    MOV {from: Storage, to: Storage, size: MemorySize},

    /// Finds the address of `from` and puts a pointer to it in the eightbyte `to`
    LEA {from: Storage, to: Storage},

    /// Compare `lhs` and `rhs` using the appropriate comparison
    CMP {lhs: Storage, rhs: Storage, data_type: ScalarType},

    /// based on the comparison, sets `storage` to 1 or 0
    SETCC {to: Storage, data_type: ScalarType, comparison: AsmComparison},

    ///based on the comparison, conditionally jump to the label
    JMPCC {label: Label, comparison: AsmComparison},

    //casts and moves from -> to
    CAST {from: Storage, from_type: ScalarType, to: Storage, to_type: ScalarType},

    /// Sums lhs and rhs, storing the result in `to`
    ADD {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    ///subtracts rhs from lhs, storing the result in `to`
    SUB {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    ///multiplies lhs by rhs and stores the result in `to`
    MUL {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},
    /// Divides lhs by rhs and stores the result in `to`
    DIV {lhs: Storage, rhs: Storage, to: Storage, data_type: ScalarType},

    ///shifts `from` logically left, storing the result in `to`
    SHL {from: Storage, from_type: IntegerType, amount: Storage, amount_type: IntegerType, to: Storage},
    ///shifts `from` logically left, storing the result in `to` (arithmetic or logical based on the signedness of `from_type`)
    SHR {from: Storage, from_type: IntegerType, amount: Storage, amount_type: IntegerType, to: Storage},

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
    Return,
    /// Subtracts MemorySize bytes from RSP
    AllocateStack(MemorySize),
    /// adds MemorySize bytes to RSP
    DeallocateStack(MemorySize),
    ///calls a subroutine (you must handle the parameters though)
    CALL {label: String},

    ///not even a nop, just a blank line of assembly
    BLANK,
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
    pub fn to_text(&self, stack: &BakedSimpleStackFrame) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => {
                let mut in_stream = MovStream::new(GPRegister::_AX, from, *size, stack);

                in_stream
                .map(|MovStreamData { put_in_reg_asm, reg_size, offset_from_start }| {
                    
                });

                todo!()
            },
            AsmOperation::LEA { from } => format!("lea {}, {}", GPRegister::acc().generate_name(PTR_SIZE), from.generate_name(stack)),
            AsmOperation::CMP { rhs, data_type } => instruction_cmp(rhs, data_type, stack),
            AsmOperation::SETCC { comparison } => instruction_setcc(comparison),
            AsmOperation::JMPCC { label, comparison } => format!("{} {}", instruction_jmpcc(comparison), label),
            AsmOperation::ADD { increment, data_type } => instruction_add(increment, data_type, stack),
            AsmOperation::SUB { decrement, data_type } => instruction_sub(decrement, data_type, stack),
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

fn instruction_mov(to: &RegOrMem, from: &Operand, size: MemorySize, stack: &BakedSimpleStackFrame) -> String {
    match (from, to) {
        (Operand::MMReg(from_reg), RegOrMem::MMReg(to_reg)) => format!("movaps {}, {}", to_reg.generate_name(size), from_reg.generate_name(size)),
        (Operand::MMReg(from_reg), RegOrMem::Mem(to_mem)) => match size.size_bytes() {
            4 => format!("movss {}, {}", to_mem.generate_name(stack), from_reg.generate_name(size)),
            8 => format!("movsd {}, {}", to_mem.generate_name(stack), from_reg.generate_name(size)),
            _ => panic!("invalid size for XMM -> RAM move")
        },
        (Operand::Mem(from_mem), RegOrMem::MMReg(to_reg)) => match size.size_bytes() {
            4 => format!("movss {}, {}", to_reg.generate_name(size), from_mem.generate_name(stack)),
            8 => format!("movsd {}, {}", to_reg.generate_name(size), from_mem.generate_name(stack)),
            _ => panic!("invalid size for RAM -> XMM move")
        },
        //this is truly diabolical - requires stack working, but doesn't clobber any registers
        (Operand::Imm(imm), RegOrMem::MMReg(to_reg)) => format!("push {}\nmovq {}, [rsp]\nadd rsp, 8", imm.generate_name(), to_reg.generate_name(size)),
        
        //simple mov commands
        (Operand::GPReg(_), RegOrMem::GPReg(_))  |
        (Operand::GPReg(_), RegOrMem::Mem(_)) |
        (Operand::Mem(_), RegOrMem::GPReg(_)) |
        (Operand::Imm(_), RegOrMem::GPReg(_)) => format!("mov {}, {}", to.generate_name(size, stack), from.generate_name(size, stack)),
        
        //gp <--> xmm
        (Operand::GPReg(from_reg), RegOrMem::MMReg(to_reg)) => match size.size_bytes() {
            4 => format!("movd {}, {}", to_reg.generate_name(size), from_reg.generate_name(size)),
            8 => format!("movq {}, {}", to_reg.generate_name(size), from_reg.generate_name(size)),
            _ => panic!()
        },
        (Operand::MMReg(from_reg), RegOrMem::GPReg(to_reg)) => match size.size_bytes() {
            4 => format!("movd {}, {}", to_reg.generate_name(size), from_reg.generate_name(size)),
            8 => format!("movq {}, {}", to_reg.generate_name(size), from_reg.generate_name(size)),
            _ => panic!()
        },

        //invalid commands
        (Operand::Imm(_), RegOrMem::Mem(_)) => panic!("cannot move an immediate directly to memory"),
        (Operand::Mem(_), RegOrMem::Mem(_)) => panic!("memory-memory mov not supported"),
    }
}

fn instruction_cmp(rhs: &Operand, data_type: &ScalarType, stack: &BakedSimpleStackFrame) -> String {
    match data_type {
        //put acc in XMM0, rhs in XMM1, then compare
        ScalarType::Float(FloatType::F32) => format!("{}\n{}\nucomiss xmm0, xmm1", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), rhs, MemorySize::from_bytes(4), stack)),
        ScalarType::Float(FloatType::F64) => format!("{}\n{}\nucomisd xmm0, xmm1", acc_to_xmm(), instruction_mov(&RegOrMem::MMReg(MMRegister::XMM1), rhs, MemorySize::from_bytes(8), stack)),

        ScalarType::Integer(integer_type) => format!("cmp {}, {}", GPRegister::acc().generate_name(integer_type.memory_size()), rhs.generate_name(integer_type.memory_size(), stack)),
    }
}
// DataType::POINTER(_) => format!("cmp {}, {}", lhs.generate_name(PTR_SIZE), rhs.generate_name(PTR_SIZE)),//comparing pointers
            // DataType::RAW(base) if base.is_integer() => format!("cmp {}, {}", lhs.generate_name(base.get_non_struct_memory_size()), rhs.generate_name(base.get_non_struct_memory_size())),//comparing integers
            // _ => panic!("currently cannot compare this data type")

fn instruction_setcc(comparison: &AsmComparison) -> String {
    let reg_name = GPRegister::acc().generate_name(MemorySize::from_bytes(1));//setting 1 byte boolean

    let comparison_instr = match comparison {
        AsmComparison::NE => "setne",
        AsmComparison::EQ => "sete",
        AsmComparison::ALWAYS => return format!("mov {}, 1 ; unconditional set", reg_name),//for set always, there is no command, so quickly return a mov command
        AsmComparison::LE {signed} => if *signed {"setle"} else {"setbe"},
        AsmComparison::GE {signed} => if *signed {"setge"} else {"setae"},
        AsmComparison::L {signed} => if *signed {"setl"} else {"setb"},
        AsmComparison::G {signed} => if *signed {"setg"} else {"seta"},
    };

    format!("{} {}", comparison_instr, reg_name)
}
fn instruction_jmpcc(comparison: &AsmComparison) -> &str {

    match comparison {
        AsmComparison::NE => "jne",
        AsmComparison::EQ => "je",
        AsmComparison::ALWAYS => "jmp",
        AsmComparison::LE {signed}  => if *signed {"jle"} else {"jbe"},
        AsmComparison::GE {signed}  => if *signed {"jge"} else {"jae"},
        AsmComparison::L {signed}  => if *signed {"jl"} else {"jb"},
        AsmComparison::G {signed}  => if *signed {"jg"} else {"ja"},
    }
}

/// Writes instructions to sign extend the accumulator
fn sign_extend(original: &MemorySize) -> String {
    match original.size_bytes() {
        1 => format!("cbw\n{}", sign_extend(&MemorySize::from_bytes(2))),
        2 => format!("cwde\n{}", sign_extend(&MemorySize::from_bytes(4))),
        4 => format!("cdqe"),
        8 => String::new(),
        _ => panic!("tried to sign extend unknown size")
    }
}

/// Writes instructions to zero extend the accumulator
fn zero_extend(original: &MemorySize) -> &str {
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

/// Returns an instruction to move RAX to XMM0
fn acc_to_xmm() -> &'static str {
    "movq xmm0, rax"
}
/// Returns an instruction to move XMM0 to RAX
fn xmm_to_acc() -> &'static str {
    "movq rax, xmm0"
}

struct MovStreamData {
    put_in_reg_asm: String,
    reg_size: MemorySize,
    offset_from_start: MemorySize
}
struct MovStream<'a> {
    /// Which register to stream the bytes into
    register: GPRegister,
    /// Where to stream the bytes from
    from: &'a Storage,
    /// How many bytes to stream
    size: MemorySize,

    stack: &'a BakedSimpleStackFrame,

    curr_offset: MemorySize
}
impl<'a> MovStream<'a> {
    pub fn new(register: Register, from: &Storage, size: MemorySize, stack: &BakedSimpleStackFrame) -> Self {
        Self {
            register, from, size, stack,
            curr_offset: MemorySize::default(),
        }
    }
}

impl<'a> Iterator for MovStream<'a> {
    type Item = MovStreamData;

    fn next(&mut self) -> Option<Self::Item> {
        let current_offset = self.curr_offset;
        let remaining = self.size - current_offset;
        if remaining == MemorySize::new() {return None;}

        let register_size = best_reg_size(remaining);
        self.curr_offset += register_size;//skip over the register for the next iteration

        let bp_offset = match &self.from {
            //start at from location, and add(but actually subtract, as this is a negative offset)
            Storage::Stack(stack_item_key) => self.stack.get(stack_item_key).offset_from_bp - current_offset,
        };

        let mov_instr = format!("mov {}, [rbp-{}]", self.register.generate_name(register_size), bp_offset);

        Some(MovStreamData { put_in_reg_asm: mov_instr, reg_size: register_size, offset_from_start: current_offset })
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
            AsmOperation::MOV { to, from, size } => format!("{} = {} ({})", to.display_ir(), from.display_ir(), size),
            AsmOperation::BLANK => String::new(),
            AsmOperation::LEA { from, to } => format!("{} = &{}", to.display_ir(), from.display_ir()),
            AsmOperation::CMP { lhs, rhs, data_type } => format!("compare {}, {} ({})", lhs.display_ir(), rhs.display_ir(), data_type),
            AsmOperation::SETCC { to, data_type, comparison } => format!("set-{} {} ({})", comparison, to.display_ir(), data_type),
            AsmOperation::JMPCC { label, comparison } => format!("jump-{} to {}", comparison, label),
            AsmOperation::CAST { from, from_type, to, to_type } => format!("cast {} -> {} ({} = {})", from_type, to_type, to, from),
            AsmOperation::ADD { lhs, rhs, to, data_type } => format!("{} = {} + {} ({})", to, lhs, rhs, data_type),
            AsmOperation::SUB { lhs, rhs, to, data_type } => format!("{} = {} - {} ({})", to, lhs, rhs, data_type),
            AsmOperation::MUL { lhs, rhs, to, data_type } => format!("{} = {} * {} ({})", to, lhs, rhs, data_type),
            AsmOperation::DIV { lhs, rhs, to, data_type } => format!("{} = {} / {} ({})", to, lhs, rhs, data_type),
            AsmOperation::SHL { from, from_type, amount, amount_type: _, to } => format!("{} = {} << {} ({})", to, from, amount, from_type),
            AsmOperation::SHR { from, from_type, amount, amount_type: _, to } => format!("{} = {} >> {} ({})", to, from, amount, from_type),
            AsmOperation::NEG { from, to, data_type } => format!("{} = -{} ({})", to, from, data_type),
            AsmOperation::BitwiseNot { from, to, size } => format!("{} = ~{} ({})", to, from, size),
            AsmOperation::BitwiseOp { lhs, rhs, to, size, operation } => format!("{} = {} {} {} ({})", to, lhs, operation, rhs, size),
            AsmOperation::Label(label) => format!("{}", label),
            AsmOperation::CreateStackFrame => format!("create stack frame and reserve stack space"),
            AsmOperation::DestroyStackFrame => format!("destroy stack frame"),
            AsmOperation::Return => format!("return"),
            AsmOperation::AllocateStack(memory_size) => format!("reserve stack ({})", memory_size),
            AsmOperation::DeallocateStack(memory_size) => format!("deallocate stack ({})", memory_size),
            AsmOperation::CALL { label } => format!("call {}", label),
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

