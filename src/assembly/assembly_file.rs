use super::assembly::Assembly;


pub struct AssemblyFile {
    string_literal_lines: Vec<String>,//raw assembly lines defining string literals
    global_func_lines: Vec<String>,//raw assembly lines exporting global functions
    extern_func_lines: Vec<String>,//raw assembly lines importing extern functions

    global_variable_lines: Vec<String>,//raw assembly lines defining and initialising global variables

    functions: Vec<Assembly>,//list of each function
}

