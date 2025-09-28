
use stack_management::{baked_stack_frame::BakedSimpleStackFrame, simple_stack_frame::SimpleStackFrame};

use crate::assembly::operand::STACK_ALIGN;

use super::assembly::IRCode;


pub struct AssemblyFile {
    string_literal_lines: Vec<String>,//raw assembly lines defining string literals
    global_labels: Vec<String>,//function names that are exported
    extern_labels: Vec<String>,//function names that are imported
    global_variable_init: Vec<String>,//initialise static and auto variables
    functions: Vec<(IRCode, SimpleStackFrame)>,//list of each function
}

impl AssemblyFile {
    pub fn builder() -> AssemblyFileBuilder {
        AssemblyFileBuilder::default()
    }

    pub fn to_nasm_file(&self) -> String {
        let global_label_text: String = self.global_labels
            .iter()
            .map(|label| format!("global {}\n", label))
            .collect();

        let extern_label_text: String = self.extern_labels
            .iter()
            .map(|label| format!("extern {}\n", label))
            .collect();

        let string_literals = self.string_literal_lines.join("\n");

        let var_init = self.global_variable_init.join("\n");

        let instructions = self.functions
            .iter()
            .map(|(asm, stack)| {
                let stack = BakedSimpleStackFrame::new(stack, STACK_ALIGN);
                asm.get_lines()
                .iter()
                .map(|x| x.emit_assembly(&stack))
                .collect::<Vec<_>>()
            })
            .flatten()//flatten each assembly's lines into a massive iterator
            .collect::<Vec<_>>()//get each line
            .join("\n");

        format!(
"
{}
{}
SECTION .rodata
align 16
FLOAT_NEGATE dd 0x80000000, 0, 0, 0
align 16
DOUBLE_NEGATE dq 0x8000000000000000, 0

{}
SECTION .data
{}
SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}",global_label_text, extern_label_text, string_literals, var_init, instructions)
    }
}

#[derive(Default)]//adds ::default() which sets all vectors to empty
pub struct AssemblyFileBuilder {
    string_literal_lines: Vec<String>,
    ///labels that must be marked global to be exported
    global_label_lines: Vec<String>,
    ///labels that must be marked extern to be imported
    extern_label_lines: Vec<String>,

    /// assembly lines for initialising static or auto variables
    global_variable_init: Vec<String>,
    functions: Vec<(IRCode, SimpleStackFrame)>,
}

impl AssemblyFileBuilder {
    pub fn string_literal_lines(mut self, lines: Vec<String>) -> Self {
        self.string_literal_lines = lines;
        self
    }

    pub fn global_label_lines(mut self, lines: Vec<String>) -> Self {
        self.global_label_lines = lines;
        self
    }

    pub fn extern_label_lines(mut self, lines: Vec<String>) -> Self {
        self.extern_label_lines = lines;
        self
    }

    pub fn functions(mut self, functions: Vec<(IRCode, SimpleStackFrame)>) -> Self {
        self.functions = functions;
        self
    }
    
    pub fn global_variable_init(mut self, inits: Vec<String>) -> Self {
        self.global_variable_init = inits;
        self
    }

    pub fn build(self) -> AssemblyFile {
        AssemblyFile {
            string_literal_lines: self.string_literal_lines,
            global_labels: self.global_label_lines,
            extern_labels: self.extern_label_lines,
            global_variable_init: self.global_variable_init,
            functions: self.functions,
        }
    }
}