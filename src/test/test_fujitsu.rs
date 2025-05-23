/**
 * tests using the test suite from https://github.com/fujitsu/compiler-test-suite/tree/main
 */

use std::{fs, path::PathBuf, process::{Command, Stdio}, str::FromStr};

use unwrap_let::unwrap_let;

use crate::compile;

use super::file_tools::{find_c_files, find_folders};

#[test]
#[ignore = "not yet implemented"]
fn test_all() {
    let test_folder = PathBuf::from_str("tests/fujitsu_testsuite").unwrap();

    let output_filename = test_folder.join("test_output.out");

    for subfolder_path in find_folders(&test_folder) {
        for c_file_path in find_c_files(&subfolder_path) {

            let expected_output_path = c_file_path.with_extension("reference_output");
            let expected_output_contents = fs::read_to_string(expected_output_path).unwrap();//read the output file
            let (expected_stdout, last_line) = expected_output_contents.trim_end().rsplit_once("\n").unwrap();//get the stdout expected, and the last line which specifies the return code
            unwrap_let!( ("exit", exit_code) = last_line.split_once(" ").unwrap());//should be in the form "exit n" so split by space and verify it starts with "exit"
    
            compile::compile(&c_file_path, &output_filename, &[]).unwrap();
    
            let binary_process = Command::new(&output_filename)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("Failed to run the compiled binary");
    
            let binary_command = binary_process
                .wait_with_output()
                .expect("Failed to run test case");
    
            println!("testing results for {:?}", c_file_path.file_name().unwrap());

            //check return code
            assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), exit_code.parse::<i32>().unwrap());
            //check stdout
            assert_eq!(String::from_utf8_lossy(&binary_command.stdout), expected_stdout);
    
        }
    }

}