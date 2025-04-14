/**
 * tests using the test suite from https://github.com/c-testsuite/c-testsuite/tree/master
 */

use std::{fs, path::{Path, PathBuf}, process::{Command, Stdio}, str::FromStr};

use crate::compile;

#[derive(Debug)]
struct TestFile {
    filename: String,
    stdout: Option<String>,
}

#[test]
fn test_all() {
    let test_folder = PathBuf::from_str("tests/c_testsuite").unwrap();

    for c_file_path in find_c_files(&test_folder) {

        let expected_output_path = c_file_path.with_extension("c.expected");
        let expected_stdout = fs::read_to_string(expected_output_path).unwrap();
        let output_filename = test_folder.join("test_output.out");

        compile::compile(&c_file_path, &output_filename, &[]).unwrap();

        let binary_process = Command::new(output_filename)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Failed to run the compiled binary");

        let binary_command = binary_process
            .wait_with_output()
            .expect("Failed to run test case");

        println!("testing results for {:?}", c_file_path.file_name().unwrap());

        //test suite test cases must always return 0
        assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), 0);

        assert_eq!(String::from_utf8_lossy(&binary_command.stdout), expected_stdout);

    }
}




fn find_c_files<P: AsRef<Path>>(dir: P) -> Vec<std::path::PathBuf> {
    let mut c_files = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "c") {
            c_files.push(path);
        }
    }

    c_files
}