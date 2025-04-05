use crate::{data_type::recursive_data_type::RecursiveDataType, memory_size::MemoryLayout};

use super::operand::{Operand, PTR_SIZE};


#[derive(Clone)]
pub enum AsmOperation {
    MOV {to: Operand, from: Operand, size: MemoryLayout},//moves size bytes from -> to

    CMP {lhs: Operand, rhs: Operand, data_type: RecursiveDataType},//compares lhs and rhs, based on their data type
    SETCC {destination: Operand, comparison: AsmComparison},// based on the comparison, sets destination to 1 or 0
    JMPCC {label: String, comparison: AsmComparison},//based on the comparison, conditionally jump to the label

    SignExtendACC {old_size: MemoryLayout},//sign extends the accumulator to i64 from the old size
    ZeroExtendACC {old_size: MemoryLayout},//zero extends the accumulator to u64 from the old size

    ADD {destination: Operand, increment: Operand, data_type: RecursiveDataType},//adds increment to destination

    NEG {item: Operand, data_type: RecursiveDataType},//negates the item, taking into account its data type

    Label {name: String},
    DestroyStackFrame,
    Return,
    BLANK,//not even a nop, just a blank line of assembly
}

#[derive(Clone)]
pub enum AsmComparison {
    ALWAYS,//always jump or set to true
    NE,//not equal
    EQ,//equal
}

impl AsmOperation {
    /**
     * converts myself into a line of assembly, with no newline
     */
    pub fn to_text(&self) -> String {
        match self {
            AsmOperation::MOV { to, from, size } => format!("mov {}, {}", to.generate_name(*size), from.generate_name(*size)),
            AsmOperation::CMP { lhs, rhs, data_type } => instruction_cmp(lhs, rhs, data_type),
            AsmOperation::SETCC { destination, comparison } => instruction_setcc(destination, comparison),
            AsmOperation::JMPCC { label, comparison } => instruction_jmpcc(label, comparison),
            AsmOperation::SignExtendACC { old_size } => instruction_sign_extend(old_size),
            AsmOperation::ZeroExtendACC { old_size } => instruction_zero_extend(old_size),
            AsmOperation::ADD { destination, increment, data_type } => instruction_add(destination, increment, data_type),
            AsmOperation::NEG { item, data_type } => instruction_neg(item, data_type),
            AsmOperation::DestroyStackFrame => "mov rsp, rbp\npop rbp".to_string(),
            AsmOperation::Return => "ret".to_string(),
            AsmOperation::Label { name } => name.clone(),
            AsmOperation::BLANK => String::new(),
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
    };

    format!("{} {}", comparison_instr, reg_name)
}
fn instruction_jmpcc(label: &str, comparison: &AsmComparison) -> String {

    let comparison_instr = match comparison {
        AsmComparison::NE => "jne",
        AsmComparison::EQ => "je",
        AsmComparison::ALWAYS => "jmp",
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

fn instruction_neg(destination: &Operand, data_type: &RecursiveDataType) -> String {
    match data_type {
        RecursiveDataType::RAW(base) if base.is_integer() && base.is_signed() => format!("neg {}", destination.generate_name(base.get_non_struct_memory_size())),
        RecursiveDataType::RAW(base) if base.is_unsigned() => panic!("cannot negate unsigned value"),
        _ => panic!("currently cannot negate this data type")
    }
}