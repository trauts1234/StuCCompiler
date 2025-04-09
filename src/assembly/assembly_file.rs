use super::assembly::Assembly;


pub struct AssemblyFile {
    string_literal_lines: Vec<String>,//raw assembly lines defining string literals
    global_func_lines: Vec<String>,//function names that are exported
    extern_func_lines: Vec<String>,//function names that are imported

    global_variable_lines: Vec<String>,//raw assembly lines defining and initialising global variables

    functions: Vec<Assembly>,//list of each function
}

impl AssemblyFile {
    pub fn builder() -> AssemblyFileBuilder {
        AssemblyFileBuilder::default()
    }

    pub fn to_nasm_file(&self) -> String {
        let global_function_text: String = self.global_func_lines
            .iter()
            .map(|func_name| format!("global {}\n", func_name))
            .collect();

        let extern_function_text: String = self.extern_func_lines
            .iter()
            .map(|func_name| format!("extern {}\n", func_name))
            .collect();

        let string_literals = self.string_literal_lines.join("\n");
        let global_vars = self.global_variable_lines.join("\n");

        let instructions = self.functions
            .iter()
            .map(|x| x.get_lines())
            .flatten()//flatten each assembly's lines into a massive iterator
            .map(|x| x.emit_assembly())
            .collect::<Vec<_>>()//get each line
            .join("\n");

        format!(
"
{}
{}
SECTION .rodata
{}
SECTION .data
{}
SECTION .note.GNU-stack ;disable executing the stack
SECTION .text
{}",global_function_text, extern_function_text, string_literals, global_vars, instructions)
    }
}

#[derive(Default)]//adds ::default() which sets all vectors to empty
pub struct AssemblyFileBuilder {
    string_literal_lines: Vec<String>,
    global_func_lines: Vec<String>,
    extern_func_lines: Vec<String>,
    global_variable_lines: Vec<String>,
    functions: Vec<Assembly>,
}

impl AssemblyFileBuilder {
    pub fn string_literal_lines(mut self, lines: Vec<String>) -> Self {
        self.string_literal_lines = lines;
        self
    }

    pub fn global_func_lines(mut self, lines: Vec<String>) -> Self {
        self.global_func_lines = lines;
        self
    }

    pub fn extern_func_lines(mut self, lines: Vec<String>) -> Self {
        self.extern_func_lines = lines;
        self
    }

    pub fn global_variable_lines(mut self, lines: Vec<String>) -> Self {
        self.global_variable_lines = lines;
        self
    }

    pub fn functions(mut self, functions: Vec<Assembly>) -> Self {
        self.functions = functions;
        self
    }

    pub fn build(self) -> AssemblyFile {
        AssemblyFile {
            string_literal_lines: self.string_literal_lines,
            global_func_lines: self.global_func_lines,
            extern_func_lines: self.extern_func_lines,
            global_variable_lines: self.global_variable_lines,
            functions: self.functions,
        }
    }
}