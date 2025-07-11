use crate::{assembly::{comparison::AsmComparison, operand::{memory_operand::MemoryOperand, register::{GPRegister, MMRegister, Register}}}, data_type::{base_type::{BaseType, FloatType, IntegerType, ScalarType}, recursive_data_type::DataType}, debugging::IRDisplay};
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

    ///compares lhs and rhs, based on their data type
    CMP {lhs: Operand, rhs: Operand, data_type: ScalarType},
    /// based on the comparison, sets destination to 1 or 0
    SETCC {destination: GPRegister, comparison: AsmComparison},
    ///based on the comparison, conditionally jump to the label
    JMPCC {label: String, comparison: AsmComparison},

    ///sign extends the accumulator to i64 from the old size
    SignExtendACC {old_size: MemorySize},
    ///zero extends the accumulator to u64 from the old size
    ZeroExtendACC {old_size: MemorySize},

    /// Cast a 64 bit GP register to a MMX float/double
    GP64CastMMX {from:GPRegister, to:MMRegister, from_is_signed: bool, to_type: FloatType },
    /// Cast a MMX float/double to 64 bit GP register
    MMXCastGP64 {from:MMRegister, to:GPRegister, from_type: FloatType, to_is_signed: bool },
    /// Cast a float to double or vice-versa
    MMXCastMMX {from:MMRegister, to:MMRegister, from_type: FloatType, to_type: FloatType },

    ///adds increment to destination
    ADD {destination: RegOrMem, increment: Operand, data_type: DataType},
    ///subtracts decrement from _AX
    SUB {decrement: Operand, data_type: DataType},
    ///multiplies _AX by the multiplier. depending on data type, injects mul or imul commands
    MUL {multiplier: RegOrMem, data_type: DataType},
    ///divides _AX by the divisor. depending on data type, injects div or idiv commands
    DIV {divisor: RegOrMem, data_type: ScalarType},
    ///shifts logically left
    SHL {destination: RegOrMem, amount: Operand, base_type: BaseType},
    ///shifts right, (arithmetic or logical based on the signedness of base_type)
    SHR {destination: RegOrMem, amount: Operand, base_type: BaseType},

    ///negates the item, taking into account its data type
    NEG {item: GPRegister, data_type: ScalarType},
    ///performs bitwise not to the item
    BitwiseNot {item: RegOrMem, size: MemorySize},

    /// applies operation to destination and secondary, saving results to destination
    BitwiseOp {destination: RegOrMem, secondary: Operand, operation: LogicalOperation, size: MemorySize},

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
            AsmOperation::CMP { lhs, rhs, data_type } => instruction_cmp(lhs, rhs, data_type),
            AsmOperation::SETCC { destination, comparison } => instruction_setcc(destination, comparison),
            AsmOperation::JMPCC { label, comparison } => format!("{} {}", instruction_jmpcc(comparison), label),
            AsmOperation::SignExtendACC { old_size } => instruction_sign_extend(old_size),
            AsmOperation::ZeroExtendACC { old_size } => instruction_zero_extend(old_size),
            AsmOperation::ADD { destination, increment, data_type } => instruction_add(destination, increment, data_type),
            AsmOperation::SUB { decrement, data_type } => instruction_sub(decrement, data_type),
            AsmOperation::NEG { item, data_type } => instruction_neg(item, data_type),
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
            AsmOperation::BitwiseOp { destination, secondary, operation, size } => instruction_bitwise(destination, secondary, operation, *size),
            AsmOperation::CALL { label } => format!("call {}", label),
            AsmOperation::SHL { destination, amount, base_type } => instruction_shiftleft(destination, amount, base_type),
            AsmOperation::SHR { destination, amount, base_type } => instruction_shiftright(destination, amount, base_type),
            AsmOperation::BitwiseNot { item, size } => format!("not {}", item.generate_name(*size)),

            AsmOperation::MMXCastMMX { from, to, from_type, to_type } => instruction_mmx_cast_mmx(from, to, from_type, to_type),
            AsmOperation::GP64CastMMX { from, to, from_is_signed, to_type } => instruction_gp64_cast_mmx(from, to, *from_is_signed, to_type),
            AsmOperation::MMXCastGP64 { from, to, from_type, to_is_signed } => todo!(),
        }
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

fn instruction_cmp(lhs: &Operand, rhs: &Operand, data_type: &ScalarType) -> String {
    match (lhs, rhs) {
        (Operand::MMReg(x), y) => todo!(),
        (x, Operand::MMReg(y)) => todo!(),

        _ => match data_type {
            ScalarType::Float(_) => panic!("tried to float-compare some integers?"),
            ScalarType::Integer(integer_type) => format!("cmp {}, {}", lhs.generate_name(integer_type.memory_size()), rhs.generate_name(integer_type.memory_size())),
        }
    }
}
// DataType::POINTER(_) => format!("cmp {}, {}", lhs.generate_name(PTR_SIZE), rhs.generate_name(PTR_SIZE)),//comparing pointers
            // DataType::RAW(base) if base.is_integer() => format!("cmp {}, {}", lhs.generate_name(base.get_non_struct_memory_size()), rhs.generate_name(base.get_non_struct_memory_size())),//comparing integers
            // _ => panic!("currently cannot compare this data type")

fn instruction_setcc(destination: &GPRegister, comparison: &AsmComparison) -> String {
    let reg_name = destination.generate_name(MemorySize::from_bytes(1));//setting 1 byte boolean

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

fn instruction_sign_extend(original: &MemorySize) -> String {
    match original.size_bytes() {
        1 => format!("cbw\n{}", instruction_sign_extend(&MemorySize::from_bytes(2))),
        2 => format!("cwde\n{}", instruction_sign_extend(&MemorySize::from_bytes(4))),
        4 => format!("cdqe\n"),
        _ => panic!("tried to sign extend unknown size")
    }
}

fn instruction_zero_extend(original: &MemorySize) -> String {
    match original.size_bytes() {
        1 => String::from("movzx rax, al\n"),
        2 => String::from("movzx rax, ax\n"),
        4 => String::new(), // Writing to EAX automatically zeroes RAX's upper half.
        _ => panic!("tried to zero extend unknown size")
    }
}

fn instruction_add(destination: &RegOrMem, increment: &Operand, data_type: &DataType) -> String {
    match data_type {
        DataType::POINTER(_) => format!("add {}, {}", destination.generate_name(PTR_SIZE), increment.generate_name(PTR_SIZE)),
        //addition is same for signed and unsigned
        DataType::RAW(base) if base.is_integer() => format!("add {}, {}", destination.generate_name(base.get_non_struct_memory_size()), increment.generate_name(base.get_non_struct_memory_size())),
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

fn instruction_neg(destination: &GPRegister, data_type: &ScalarType) -> String {
    match data_type {
        ScalarType::Float(FloatType::F32) => format!("xorps {}, [FLOAT_NEGATE]", destination.generate_name(MemorySize::from_bytes(4))),
        ScalarType::Float(FloatType::F64) => todo!(),
        ScalarType::Integer(integer_type) => format!("neg {}", destination.generate_name(integer_type.memory_size())),
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

fn instruction_bitwise(destination: &RegOrMem, secondary: &Operand, operation: &LogicalOperation, size: MemorySize) -> String {
    let op_asm = match operation {
        LogicalOperation::AND => "and".to_string(),
        LogicalOperation::OR => "or".to_string(),
        LogicalOperation::XOR => "xor".to_string()
    };

    format!("{} {}, {}", op_asm, destination.generate_name(size), secondary.generate_name(size))
}

fn instruction_shiftleft(destination: &RegOrMem, amount: &Operand, base_type: &BaseType) -> String {
    let size = base_type.get_non_struct_memory_size();
    format!("shl {}, {}", destination.generate_name(size), amount.generate_name(MemorySize::from_bytes(1)))
}

fn instruction_shiftright(destination: &RegOrMem, amount: &Operand, base_type: &BaseType) -> String {
    let size = base_type.get_non_struct_memory_size();
    match base_type {
        //signed shift needs algebraic shift right
        base if base.is_signed() => format!("sar {}, {}", destination.generate_name(size), amount.generate_name(MemorySize::from_bytes(1))),
        //unsigned uses logical shift
        base if base.is_unsigned() => format!("shr {}, {}", destination.generate_name(size), amount.generate_name(MemorySize::from_bytes(1))),
        _ => panic!("cannot shift this type")
    }
}


macro_rules! opcode {
    ($op:expr) => {
        ($op.yellow().to_string())
    }
}

impl IRDisplay for AsmOperation {
    fn display_ir(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => format!("{} = {} ({})", to.display_ir(), from.display_ir(), size),
            AsmOperation::LEA { from } => format!("{} = {} {}", GPRegister::acc().display_ir(), opcode!("LEA"), from.display_ir()),
            AsmOperation::CMP { lhs, rhs, data_type } => 
                        format!("{} {}, {} ({})",
                            opcode!("CMP"),
                            lhs.display_ir(),
                            rhs.display_ir(),
                            data_type
                        ),
            AsmOperation::SETCC { destination, comparison } => 
                        format!("{} {}",
                            opcode!(format!("set-{}", comparison.display_ir())),
                            destination.display_ir()
                        ),
            AsmOperation::JMPCC { label, comparison } => 
                        format!("{} {}",
                            opcode!(format!("jmp-{}", comparison.display_ir())),
                            label
                        ),
            AsmOperation::SignExtendACC { old_size } => format!("{} {} from {}", opcode!("sign extend"), GPRegister::acc().display_ir(), old_size),
            AsmOperation::ZeroExtendACC { old_size } => format!("{} {} from {}", opcode!("zero extend"), GPRegister::acc().display_ir(), old_size),
            AsmOperation::ADD { destination, increment, data_type } => format!("{} += {} ({})", destination.display_ir(), increment.display_ir(), data_type),
            AsmOperation::SUB { decrement, data_type } => format!("{} -= {} ({})", GPRegister::acc().display_ir(), decrement.display_ir(), data_type),
            AsmOperation::MUL { multiplier, data_type } => format!("{} *= {} ({})", GPRegister::acc().display_ir(), multiplier.display_ir(), data_type),
            AsmOperation::DIV { divisor, data_type } => format!("{} /= {} ({})", GPRegister::acc().display_ir(), divisor.display_ir(), data_type),
            AsmOperation::SHL { destination, amount, base_type } => format!("{} <<= {} ({})", destination.display_ir(), amount.display_ir(), base_type),
            AsmOperation::SHR { destination, amount, base_type } => format!("{} >>= {} ({})", destination.display_ir(), amount.display_ir(), base_type),
            AsmOperation::NEG { item, data_type } => format!("{} {} ({})", opcode!("NEG"), item.display_ir(), data_type),
            AsmOperation::BitwiseNot { item, size } => format!("{} {} ({})", opcode!("NOT"), item.display_ir(), size),
            AsmOperation::BitwiseOp { destination, secondary, operation, size } => format!("{} {} {} ({})", destination.display_ir(), operation.display_ir(), secondary.display_ir(), size),
            AsmOperation::Label { name } => format!("{}:", name.red().to_string()),
            AsmOperation::CreateStackFrame => opcode!("CreateStackFrame"),
            AsmOperation::DestroyStackFrame => opcode!("DestroyStackFrame"),
            AsmOperation::Return => opcode!("RET"),
            AsmOperation::AllocateStack(size) => format!("{} {} B", opcode!("reserve stack"), size.size_bytes()),
            AsmOperation::DeallocateStack(size) => format!("{} {} B", opcode!("deallocate stack"), size.size_bytes()),
            AsmOperation::MEMCPY { size } => format!("{} {}", opcode!("MEMCPY"), size),
            AsmOperation::CALL { label } => format!("{} {}", opcode!("CALL"), label),
            AsmOperation::BLANK => String::new(),

            AsmOperation::GP64CastMMX { from, to, from_is_signed, to_type } => format!("{} = ({})({}){}", to.display_ir(), to_type, if *from_is_signed {"i64"} else {"u64"}, from.display_ir()),
            AsmOperation::MMXCastGP64 { from, to, from_type, to_is_signed } => format!("{} = ({})({}){}", to.display_ir(), if *to_is_signed {"i64"} else {"u64"}, from_type, from.display_ir()),
            AsmOperation::MMXCastMMX { from, to, from_type, to_type } => format!("{} = ({})({}){}", to.display_ir(), to_type, from_type, from.display_ir()),
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

