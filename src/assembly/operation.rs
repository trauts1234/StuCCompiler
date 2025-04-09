use crate::{data_type::{base_type::BaseType, recursive_data_type::RecursiveDataType}, memory_size::MemoryLayout};

use super::operand::{Operand, PTR_SIZE};


#[derive(Clone)]
pub enum AsmOperation {
    ///moves size bytes from -> to
    MOV {to: Operand, from: Operand, size: MemoryLayout},
    ///references from, puts address in to
    LEA {to: Operand, from: Operand},

    ///compares lhs and rhs, based on their data type
    CMP {lhs: Operand, rhs: Operand, data_type: RecursiveDataType},
    /// based on the comparison, sets destination to 1 or 0
    SETCC {destination: Operand, comparison: AsmComparison},
    ///based on the comparison, conditionally jump to the label
    JMPCC {label: String, comparison: AsmComparison},

    ///sign extends the accumulator to i64 from the old size
    SignExtendACC {old_size: MemoryLayout},
    ///zero extends the accumulator to u64 from the old size
    ZeroExtendACC {old_size: MemoryLayout},

    ///adds increment to destination
    ADD {destination: Operand, increment: Operand, data_type: RecursiveDataType},
    ///subtracts decrement from destination
    SUB {destination: Operand, decrement: Operand, data_type: RecursiveDataType},
    ///multiplies _AX by the multiplier. depending on data type, injects mul or imul commands
    MUL {multiplier: Operand, data_type: RecursiveDataType},
    ///divides _AX by the divisor. depending on data type, injects div or idiv commands
    DIV {divisor: Operand, data_type: RecursiveDataType},

    ///negates the item, taking into account its data type
    NEG {item: Operand, data_type: RecursiveDataType},

    /// applies operation to destination and secondary, saving results to destination
    BooleanOp {destination: Operand, secondary: Operand, operation: AsmBooleanOperation},

    Label {name: String},
    CreateStackFrame,
    DestroyStackFrame,
    Return,
    ///copies size bytes from the pointer RDI to RSI
    MEMCPY {size: MemoryLayout},
    ///calls a subroutine
    CALL {label: String},
    ///not even a nop, just a blank line of assembly
    BLANK,

    Pop64 {destination: Operand},//not used much
}

#[derive(Clone)]
pub enum AsmComparison {
    ALWAYS,//always jump or set to true
    NE,//not equal
    EQ,//equal
    ///less than or equal to
    LE,
    ///greater than or equal to
    GE,
    ///less than
    L,
    ///greater than
    G,
}

#[derive(Clone)]
pub enum AsmBooleanOperation {
    AND,
    OR,
}

impl AsmOperation {
    /**
     * converts myself into a line of assembly, with no newline
     */
    pub fn to_text(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => format!("mov {}, {}", to.generate_name(*size), from.generate_name(*size)),
            AsmOperation::LEA { to, from } => format!("lea {}, {}", to.generate_name(PTR_SIZE), from.generate_name(PTR_SIZE)),
            AsmOperation::CMP { lhs, rhs, data_type } => instruction_cmp(lhs, rhs, data_type),
            AsmOperation::SETCC { destination, comparison } => instruction_setcc(destination, comparison),
            AsmOperation::JMPCC { label, comparison } => instruction_jmpcc(label, comparison),
            AsmOperation::SignExtendACC { old_size } => instruction_sign_extend(old_size),
            AsmOperation::ZeroExtendACC { old_size } => instruction_zero_extend(old_size),
            AsmOperation::ADD { destination, increment, data_type } => instruction_add(destination, increment, data_type),
            AsmOperation::SUB { destination, decrement, data_type } => instruction_sub(destination, decrement, data_type),
            AsmOperation::NEG { item, data_type } => instruction_neg(item, data_type),
            AsmOperation::CreateStackFrame => "push rbp\nmov rbp, rsp".to_string(),
            AsmOperation::DestroyStackFrame => "mov rsp, rbp\npop rbp".to_string(),
            AsmOperation::Return => "ret".to_string(),
            AsmOperation::Label { name } => format!("{}:", name),
            AsmOperation::MEMCPY { size } => format!("mov rcx, {}\ncld\nrep movsb", size.size_bytes()),
            AsmOperation::BLANK => String::new(),
            AsmOperation::MUL { multiplier, data_type } => instruction_mul(multiplier, data_type),
            AsmOperation::DIV { divisor, data_type } => instruction_div(divisor, data_type),
            AsmOperation::BooleanOp { destination, secondary, operation } => instruction_boolean(destination, secondary, operation),
            AsmOperation::Pop64 { destination } => format!("pop {}", destination.generate_name(PTR_SIZE)),
            AsmOperation::CALL { label } => format!("call {}", label),
        }
    }
}

fn instruction_cmp(lhs: &Operand, rhs: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::POINTER(_) => format!("cmp {}, {}", lhs.generate_name(PTR_SIZE), rhs.generate_name(PTR_SIZE)),//comparing pointers
        RecursiveDataType::RAW(base) if base.is_integer() => format!("cmp {}, {}", lhs.generate_name(base.get_non_struct_memory_size()), rhs.generate_name(base.get_non_struct_memory_size())),//comparing integers
        _ => panic!("currently cannot compare this data type")
    }
}

fn instruction_setcc(destination: &Operand, comparison: &AsmComparison) -> String {
    let reg_name = destination.generate_name(MemoryLayout::from_bytes(1));//setting 1 byte boolean

    let comparison_instr = match comparison {
        AsmComparison::NE => "setne",
        AsmComparison::EQ => "seteq",
        AsmComparison::ALWAYS => todo!("unconditional set register to 1"),
        AsmComparison::LE => "setle",
        AsmComparison::GE => "setge",
        AsmComparison::L => "setl",
        AsmComparison::G => "setg",
    };

    format!("{} {}", comparison_instr, reg_name)
}
fn instruction_jmpcc(label: &str, comparison: &AsmComparison) -> String {

    let comparison_instr = match comparison {
        AsmComparison::NE => "jne",
        AsmComparison::EQ => "je",
        AsmComparison::ALWAYS => "jmp",
        AsmComparison::LE => "jle",
        AsmComparison::GE => "jge",
        AsmComparison::L => "jl",
        AsmComparison::G => "jg",
    };

    format!("{} {}", comparison_instr, label)
}

fn instruction_sign_extend(original: &MemoryLayout) -> String {
    match original.size_bits() {
        8 => format!("cbw\n{}", instruction_sign_extend(&MemoryLayout::from_bits(16))),
        16 => format!("cwde\n{}", instruction_sign_extend(&MemoryLayout::from_bits(32))),
        32 => format!("cdqe\n"),
        _ => panic!("tried to sign extend unknown size")
    }
}

fn instruction_zero_extend(original: &MemoryLayout) -> String {
    match original.size_bits() {
        8 => String::from("movzx rax, al\n"),
        16 => String::from("movzx rax, ax\n"),
        32 => String::new(), // Writing to EAX automatically zeroes RAX's upper half.
        _ => panic!("tried to zero extend unknown size")
    }
}

fn instruction_add(destination: &Operand, increment: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::POINTER(_) => format!("add {}, {}", destination.generate_name(PTR_SIZE), increment.generate_name(PTR_SIZE)),
        //addition is same for signed and unsigned
        RecursiveDataType::RAW(base) if base.is_integer() => format!("add {}, {}", destination.generate_name(base.get_non_struct_memory_size()), increment.generate_name(base.get_non_struct_memory_size())),
        _ => panic!("currently cannot add this data type")
    }
}
fn instruction_sub(destination: &Operand, decrement: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::POINTER(_) => format!("sub {}, {}", destination.generate_name(PTR_SIZE), decrement.generate_name(PTR_SIZE)),
        //subtraction is same for signed and unsigned
        RecursiveDataType::RAW(base) if base.is_integer() => format!("sub {}, {}", destination.generate_name(base.get_non_struct_memory_size()), decrement.generate_name(base.get_non_struct_memory_size())),
        _ => panic!("currently cannot sub this data type")
    }
}

fn instruction_neg(destination: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::RAW(base) if base.is_integer() && base.is_signed() => format!("neg {}", destination.generate_name(base.get_non_struct_memory_size())),
        RecursiveDataType::RAW(base) if base.is_unsigned() => panic!("cannot negate unsigned value"),
        _ => panic!("currently cannot negate this data type")
    }
}

fn instruction_div(divisor: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::RAW(BaseType::I32) => format!("cdq\nidiv {}", divisor.generate_name(MemoryLayout::from_bits(32))),
        RecursiveDataType::RAW(BaseType::I64) => format!("cqo\nidiv {}", divisor.generate_name(MemoryLayout::from_bits(64))),
        _ => panic!("cannot divide by this type")
    }
}

fn instruction_mul(multiplier: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::RAW(base) if base.is_signed() => format!("imul {}", multiplier.generate_name(base.get_non_struct_memory_size())),
        RecursiveDataType::ARRAY {..} => panic!("cannot multiply an array"),
        RecursiveDataType::POINTER(_) => panic!("cannot multiply pointer"),
        _ => panic!("unsupported data type")
    }
}

fn instruction_boolean(destination: &Operand, secondary: &Operand, operation: &AsmBooleanOperation) -> String {
    let op_asm = match operation {
        AsmBooleanOperation::AND => "and".to_string(),
        AsmBooleanOperation::OR => "or".to_string(),
    };

    let bool_size = MemoryLayout::from_bytes(1);

    format!("{} {}, {}", op_asm, destination.generate_name(bool_size), secondary.generate_name(bool_size))
}