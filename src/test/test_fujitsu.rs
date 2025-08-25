/**
 * tests using the test suite from https://github.com/fujitsu/compiler-test-suite/tree/main
 */

use std::{fs::{self, OpenOptions}, io::Write, path::PathBuf, process::{Command, Stdio}, str::FromStr};

use unwrap_let::unwrap_let;

use crate::compile;

use super::file_tools::{find_c_files, find_folders};

struct PassedTests {
    paths: Vec<PathBuf>
}
impl PassedTests {
    pub fn new(file_text: String) -> Self {
        let mut paths = Vec::new();
        for line in file_text.lines() {
            paths.push(line.try_into().unwrap());
        }
        Self{paths}
    }
    pub fn previously_passed(&self, path: &PathBuf) -> bool {
        self.paths.contains(path)
    }
}

#[test]
// #[ignore = "not yet implemented"]
fn test_all() {
    let test_folder = PathBuf::from_str("tests/fujitsu_testsuite").unwrap();

    let output_filename = test_folder.join("test_output.out");

    let success_paths_filename = test_folder.join("successful_tests.txt");
    let previous_passes = PassedTests::new(fs::read_to_string(success_paths_filename.clone()).unwrap());//keeps track of previously passed tests
    //write filenames of new passes
    let mut new_passes = OpenOptions::new()
        .append(true)
        .open(success_paths_filename)
        .unwrap();

    for subfolder_path in find_folders(&test_folder) {
        for c_file_path in find_c_files(&subfolder_path) {

            let expected_output_path = c_file_path.with_extension("reference_output");
            let expected_output = fs::read_to_string(expected_output_path);//try to read the output
            //if there is an output, return code and stdout code
            let expected_output: Option<(i32, &str)> = match &expected_output {
                Ok(text) => {
                    let (expected_stdout, last_line) = text.trim_end().rsplit_once("\n").unwrap();//get the stdout expected, and the last line which specifies the return code
                    unwrap_let!( ("exit", exit_code) = last_line.split_once(" ").unwrap());//should be in the form "exit n" so split by space and verify it starts with "exit"
                    Some((exit_code.parse::<i32>().unwrap(), expected_stdout))
                },

                Err(_) => None
            };

            compile::compile(&c_file_path, &output_filename, &[], true).unwrap();
    
            let binary_process = Command::new(&output_filename)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to run the compiled binary");
    
            let binary_command = binary_process
                .wait_with_output()
                .expect("Failed to run test case");

            let prev_passed_msg = if previous_passes.previously_passed(&c_file_path) {"(previously passed)"} else {""};
            println!("testing results for {:?} {}", c_file_path.file_name().unwrap(), prev_passed_msg);

            //if I have some results to compare, check them
            if let Some((return_code, stdout_text)) = expected_output {
                //check return code
                assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), return_code);
                //check stdout
                assert_eq!(String::from_utf8_lossy(&binary_command.stdout).trim_end_matches("\n"), stdout_text);
            }

            if !previous_passes.previously_passed(&c_file_path) {
                //yay! I passed a new test
                new_passes.write_all(format!("{}\n", c_file_path.display()).as_bytes()).unwrap();
                new_passes.flush().unwrap();//just in case I inevetably crash
            }
    
        }
    }

}