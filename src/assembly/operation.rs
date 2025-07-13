use crate::{assembly::{comparison::AsmComparison, operand::{memory_operand::MemoryOperand, register::{GPRegister, MMRegister}}}, data_type::{base_type::{BaseType, FloatType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::IRDisplay};
use colored::Colorize;
use memory_size::MemorySize;
use unwrap_let::unwrap_let;
use super::{operand::{Operand, RegOrMem, PTR_SIZE}};


#[derive(Clone)]
pub enum AsmOperation {
    ///moves size bytes from -> to
    MOV {to: RegOrMem, from: Operand, size: MemorySize},
    ///references from, puts address in the accumulator
    LEA {from: MemoryOperand},

    ///compares the accumulator and rhs, based on their data type
    CMP {rhs: Operand, data_type: ScalarType},
    /// based on the comparison, sets _AX to 1 or 0
    SETCC {comparison: AsmComparison},
    ///based on the comparison, conditionally jump to the label
    JMPCC {label: String, comparison: AsmComparison},

    //casts and moves from -> to
    CAST {from_type: ScalarType, to_type: ScalarType},

    ///adds increment to _AX
    ADD {increment: Operand, data_type: DataType},
    ///subtracts decrement from _AX
    SUB {decrement: Operand, data_type: DataType},
    ///multiplies _AX by the multiplier. depending on data type, injects mul or imul commands
    MUL {multiplier: RegOrMem, data_type: DataType},
    ///divides _AX by the divisor. depending on data type, injects div or idiv commands
    DIV {divisor: RegOrMem, data_type: ScalarType},
    ///shifts logically left
    SHL {amount: Operand, base_type: BaseType},
    ///shifts right, (arithmetic or logical based on the signedness of base_type)
    SHR {amount: Operand, base_type: BaseType},

    ///negates the accumulator item, taking into account its data type
    NEG {data_type: ScalarType},
    ///performs bitwise not to the accumulator
    BitwiseNot,

    /// applies operation to destination and secondary, saving results to the accumulator
    BitwiseOp { secondary: Operand, operation: LogicalOperation},

    Label {name: String},
    CreateStackFrame,
    DestroyStackFrame,
    Return,
    /// Subtracts MemorySize bytes from RSP
    AllocateStack(MemorySize),
    /// adds MemorySize bytes to RSP
    DeallocateStack(MemorySize),
    ///copies size bytes from the pointer RDI to RSI
    MEMCPY {size: MemorySize},
    ///calls a subroutine
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

impl AsmOperation {
    /**
     * converts myself into a line of assembly, with no newline
     */
    pub fn to_text(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => instruction_mov(to, from, *size),
            AsmOperation::LEA { from } => format!("lea {}, {}", GPRegister::acc().generate_name(PTR_SIZE), from.generate_name()),
            AsmOperation::CMP { rhs, data_type } => instruction_cmp(rhs, data_type),
            AsmOperation::SETCC { comparison } => instruction_setcc(comparison),
            AsmOperation::JMPCC { label, comparison } => format!("{} {}", instruction_jmpcc(comparison), label),
            AsmOperation::ADD { increment, data_type } => instruction_add(increment, data_type),
            AsmOperation::SUB { decrement, data_type } => instruction_sub(decrement, data_type),
            AsmOperation::NEG { data_type } => instruction_neg(data_type),
            AsmOperation::CreateStackFrame => "push rbp\nmov rbp, rsp".to_string(),
            AsmOperation::DestroyStackFrame => "mov rsp, rbp\npop rbp".to_string(),
            AsmOperation::Return => "ret".to_string(),
            AsmOperation::AllocateStack(size) => format!("sub rsp, {}", size.size_bytes()),
            AsmOperation::DeallocateStack(size) => format!("add rsp, {}", size.size_bytes()),
            AsmOperation::Label { name } => format!("{}:", name),
            AsmOperation::MEMCPY { size } => format!("mov rcx, {}\ncld\nrep movsb", size.size_bytes()),
            AsmOperation::BLANK => String::new(),
            AsmOperation::MUL { multiplier, data_type } => instruction_mul(multiplier, data_type),
            AsmOperation::DIV { divisor, data_type } => instruction_div(divisor, data_type),
            AsmOperation::BitwiseOp { secondary, operation} => instruction_bitwise(secondary, operation),
            AsmOperation::CALL { label } => format!("call {}", label),
            AsmOperation::SHL { amount, base_type } => instruction_shiftleft(amount, base_type),
            AsmOperation::SHR { amount, base_type } => instruction_shiftright(amount, base_type),
            AsmOperation::BitwiseNot => format!("not {}", GPRegister::acc().generate_name(MemorySize::from_bits(64))),//just NOT the whole reg

            AsmOperation::CAST { from_type, to_type } => instruction_cast(from_type, to_type)
        }
    }
}

fn instruction_cast(from_type: &ScalarType, to_type: &ScalarType) -> String {
    match (from_type, to_type) {
        (ScalarType::Integer(lhs), ScalarType::Integer(_)) => {
            let lhs_original_size = lhs.memory_size();
            //extend to 64 bits, as truncation is automatic
            if lhs.is_unsigned() {
                zero_extend(&lhs_original_size).to_string()
            } else {
                sign_extend(&lhs_original_size)
            }
        },
        _ => todo!()
    }
}

fn instruction_gp64_cast_mmx(from: &GPRegister, to: &MMRegister, from_is_signed: bool, to_type: &FloatType) -> String {
    let cast_opcode = match (from_is_signed, to_type) {
        (true, FloatType::F32) => "cvtsi2ss",//ConVert Signed Integer 2 Signed Single ?
        (true, FloatType::F64) => "cvtsi2sd",//ConVert Signed Integer 2 Signed Double ?

        (true, x) => panic!("cannot cast i64 to floating type {:?}", x),
        (false, _) => panic!("cannot currently cast u64 to float/double"),
    };

    format!(
        "{} {}, {}",
        cast_opcode,
        from.generate_name(MemorySize::from_bits(64)),
        to.generate_name(to_type.memory_size())
    )
}

fn instruction_mmx_cast_mmx(from: &MMRegister, to: &MMRegister, from_type: &FloatType, to_type: &FloatType) -> String {
    match (from_type, to_type) {
        (FloatType::F32, FloatType::F32) => String::new(),
        (FloatType::F64, FloatType::F64) => String::new(),
        (FloatType::F32, FloatType::F64) => format!("cvtss2sd {}, {}", from.generate_name(MemorySize::from_bits(32)), to.generate_name(MemorySize::from_bits(64))),
        (FloatType::F64, FloatType::F32) => format!("cvtsd2ss {}, {}", from.generate_name(MemorySize::from_bits(64)), to.generate_name(MemorySize::from_bits(32))),
    }
}

fn instruction_mov(to: &RegOrMem, from: &Operand, size: MemorySize) -> String {
    match (from, to) {
        (Operand::MMReg(from_reg), RegOrMem::MMReg(to_reg)) => todo!(),
        (Operand::MMReg(from_reg), RegOrMem::Mem(to_mem)) => todo!(),
        (Operand::Mem(from_mem), RegOrMem::MMReg(to_reg)) => todo!(),
        (Operand::Imm(imm), RegOrMem::MMReg(to_reg)) => todo!("recursively call to move via a GP register? which registers would it clobber"),
        
        //simple mov commands
        (Operand::GPReg(_), RegOrMem::GPReg(_))  |
        (Operand::GPReg(_), RegOrMem::Mem(_)) |
        (Operand::Mem(_), RegOrMem::GPReg(_)) |
        (Operand::Imm(_), RegOrMem::GPReg(_)) => format!("mov {}, {}", to.generate_name(size), from.generate_name(size)),
        
        //invalid commands
        (Operand::GPReg(_), RegOrMem::MMReg(_)) |
        (Operand::MMReg(_), RegOrMem::GPReg(_)) => todo!("bitwise mov between gp and mmx registers"),
        (Operand::Imm(_), RegOrMem::Mem(_)) => panic!("cannot move an immediate directly to memory"),
        (Operand::Mem(_), RegOrMem::Mem(_)) => panic!("memory-memory mov not supported"),
    }
}

fn instruction_cmp(rhs: &Operand, data_type: &ScalarType) -> String {
    match data_type {
        ScalarType::Float(_) => panic!("tried to float-compare some integers?"),
        ScalarType::Integer(integer_type) => format!("cmp {}, {}", GPRegister::acc().generate_name(integer_type.memory_size()), rhs.generate_name(integer_type.memory_size())),
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

/// Writes instructions (newline terminated) to sign extend the accumulator
fn sign_extend(original: &MemorySize) -> String {
    match original.size_bytes() {
        1 => format!("cbw\n{}", sign_extend(&MemorySize::from_bytes(2))),
        2 => format!("cwde\n{}", sign_extend(&MemorySize::from_bytes(4))),
        4 => format!("cdqe\n"),
        8 => String::new(),
        _ => panic!("tried to sign extend unknown size")
    }
}

/// Writes instructions (newline terminated) to zero extend the accumulator
fn zero_extend(original: &MemorySize) -> &str {
    match original.size_bytes() {
        1 => "movzx rax, al\n",
        2 => "movzx rax, ax\n",
        4 => "",// Writing to EAX automatically zeroes RAX's upper half.
        8 => "",//already right size
        _ => panic!("tried to zero extend unknown size")
    }
}

fn instruction_add(increment: &Operand, data_type: &DataType) -> String {
    match data_type {
        DataType::POINTER(_) => format!("add {}, {}", GPRegister::acc().generate_name(PTR_SIZE), increment.generate_name(PTR_SIZE)),
        //addition is same for signed and unsigned
        DataType::RAW(base) if base.is_integer() => format!("add {}, {}", GPRegister::acc().generate_name(base.get_non_struct_memory_size()), increment.generate_name(base.get_non_struct_memory_size())),
        _ => panic!("currently cannot add this data type")
    }
}
fn instruction_sub(decrement: &Operand, data_type: &DataType) -> String {
    match data_type {
        DataType::POINTER(_) => format!("sub {}, {}", GPRegister::acc().generate_name(PTR_SIZE), decrement.generate_name(PTR_SIZE)),
        //subtraction is same for signed and unsigned
        DataType::RAW(base) if base.is_integer() => format!("sub {}, {}", GPRegister::acc().generate_name(base.get_non_struct_memory_size()), decrement.generate_name(base.get_non_struct_memory_size())),
        _ => panic!("currently cannot sub this data type")
    }
}

fn instruction_neg(data_type: &ScalarType) -> String {
    match data_type {
        ScalarType::Float(FloatType::F32) => format!("xorps {}, [FLOAT_NEGATE]", MMRegister::acc().generate_name(MemorySize::from_bytes(4))),
        ScalarType::Float(FloatType::F64) => todo!(),
        ScalarType::Integer(integer_type) => format!("neg {}", GPRegister::acc().generate_name(integer_type.memory_size())),
    }
}

fn instruction_div(divisor: &RegOrMem, data_type: &ScalarType) -> String {
    match data_type {
        ScalarType::Integer(IntegerType::I32) => format!("cdq\nidiv {}", divisor.generate_name(MemorySize::from_bytes(4))),
        ScalarType::Integer(IntegerType::I64) => format!("cqo\nidiv {}", divisor.generate_name(MemorySize::from_bytes(8))),
        ScalarType::Integer(IntegerType::U32) => format!("mov edx, 0\ndiv {}", divisor.generate_name(MemorySize::from_bytes(4))),
        ScalarType::Integer(IntegerType::U64) => format!("mov rdx, 0\ndiv {}", divisor.generate_name(MemorySize::from_bytes(8))),
        x => panic!("cannot divide by this type: {}", x)
    }
}

fn instruction_mul(multiplier: &RegOrMem, data_type: &DataType) -> String {
    if let DataType::ARRAY {..} = data_type {
        panic!("cannot multiply an array");
    }
    unwrap_let!(ScalarType::Integer(base) = data_type.decay_to_primative());//float not implemented
    if base.is_unsigned() {
        format!("mul {}", multiplier.generate_name(base.memory_size()))
    } else {
        format!("imul {}", multiplier.generate_name(base.memory_size()))
    }
}

fn instruction_bitwise( secondary: &Operand, operation: &LogicalOperation) -> String {
    let op_asm = match operation {
        LogicalOperation::AND => "and".to_string(),
        LogicalOperation::OR => "or".to_string(),
        LogicalOperation::XOR => "xor".to_string()
    };

    let (primary_name, secondary_name) = match secondary {
        Operand::GPReg(gpregister) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), gpregister.generate_name(MemorySize::from_bits(64))),//TODO ensure AX is active
        Operand::MMReg(mmregister) => panic!(),
        Operand::Mem(memory_operand) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), memory_operand.generate_name()),
        Operand::Imm(immediate_value) => (GPRegister::acc().generate_name(MemorySize::from_bits(64)), immediate_value.generate_name()),
    };

    format!("{} {}, {}", op_asm, primary_name, secondary_name)
}

fn instruction_shiftleft(amount: &Operand, base_type: &BaseType) -> String {
    let size = base_type.get_non_struct_memory_size();
    format!("shl {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1)))
}

fn instruction_shiftright(amount: &Operand, base_type: &BaseType) -> String {
    let size = base_type.get_non_struct_memory_size();
    match base_type {
        //signed shift needs algebraic shift right
        base if base.is_signed() => format!("sar {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1))),
        //unsigned uses logical shift
        base if base.is_unsigned() => format!("shr {}, {}", GPRegister::acc().generate_name(size), amount.generate_name(MemorySize::from_bytes(1))),
        _ => panic!("cannot shift this type")
    }
}


macro_rules! opcode {
    ($op:expr) => {
        ($op.yellow().to_string())
    }
}
macro_rules! acc {
    () => {
        ("acc".red().to_string())
    };
}

impl IRDisplay for AsmOperation {
    fn display_ir(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => format!("{} = {} ({})", to.display_ir(), from.display_ir(), size),
            AsmOperation::LEA { from } => format!("{} = {} {}", acc!(), opcode!("LEA"), from.display_ir()),
            AsmOperation::CMP { rhs, data_type } => 
                        format!("{} {}, {} ({})",
                            opcode!("CMP"),
                            acc!(),
                            rhs.display_ir(),
                            data_type
                        ),
            AsmOperation::SETCC { comparison } => 
                        format!("{} {}",
                            opcode!(format!("set-{}", comparison.display_ir())),
                            acc!()
                        ),
            AsmOperation::JMPCC { label, comparison } => 
                        format!("{} {}",
                            opcode!(format!("jmp-{}", comparison.display_ir())),
                            label
                        ),
            AsmOperation::ADD { increment, data_type } => format!("{} += {} ({})", acc!(), increment.display_ir(), data_type),
            AsmOperation::SUB { decrement, data_type } => format!("{} -= {} ({})", acc!(), decrement.display_ir(), data_type),
            AsmOperation::MUL { multiplier, data_type } => format!("{} *= {} ({})", acc!(), multiplier.display_ir(), data_type),
            AsmOperation::DIV { divisor, data_type } => format!("{} /= {} ({})", acc!(), divisor.display_ir(), data_type),
            AsmOperation::SHL { amount, base_type } => format!("{} <<= {} ({})", acc!(), amount.display_ir(), base_type),
            AsmOperation::SHR { amount, base_type } => format!("{} >>= {} ({})", acc!(), amount.display_ir(), base_type),
            AsmOperation::NEG { data_type } => format!("{} {} ({})", opcode!("NEG"), acc!(), data_type),
            AsmOperation::BitwiseNot => format!("{} {}", opcode!("NOT"), acc!()),
            AsmOperation::BitwiseOp { secondary, operation} => format!("{} {} {}", acc!(), operation.display_ir(), secondary.display_ir()),
            AsmOperation::Label { name } => format!("{}:", name.red().to_string()),
            AsmOperation::CreateStackFrame => opcode!("CreateStackFrame"),
            AsmOperation::DestroyStackFrame => opcode!("DestroyStackFrame"),
            AsmOperation::Return => opcode!("RET"),
            AsmOperation::AllocateStack(size) => format!("{} {} B", opcode!("reserve stack"), size.size_bytes()),
            AsmOperation::DeallocateStack(size) => format!("{} {} B", opcode!("deallocate stack"), size.size_bytes()),
            AsmOperation::MEMCPY { size } => format!("{} {}", opcode!("MEMCPY"), size),
            AsmOperation::CALL { label } => format!("{} {}", opcode!("CALL"), label),
            AsmOperation::CAST { from_type, to_type } => format!("{} {} {} -> {}", opcode!("CAST"), acc!(), from_type,  to_type),
            AsmOperation::BLANK => String::new(),
        }
    }
}

impl IRDisplay for LogicalOperation {
    fn display_ir(&self) -> String {
        match self {
            LogicalOperation::AND => "&=",
            LogicalOperation::OR => "|=",
            LogicalOperation::XOR => "^=",
        }.to_owned()
    }
}

impl IRDisplay for AsmComparison {
    fn display_ir(&self) -> String {
        match self {
            AsmComparison::ALWAYS => "always",
            AsmComparison::NE => "ne",
            AsmComparison::EQ => "eq",
            AsmComparison::LE {..} => "le",
            AsmComparison::GE {..} => "ge",
            AsmComparison::L {..} => "l",
            AsmComparison::G {..} => "g",
        }.to_owned()
    }
}

