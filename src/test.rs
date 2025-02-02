
use std::{fs, process::Command};

use serde::{Deserialize, Serialize};

use crate::compile;


#[derive(Serialize, Deserialize, Debug)]
struct TestCase {
    args: Option<Vec<String>>,
    output_code: i32,
}

#[derive(Serialize, Deserialize, Debug)]
struct TestFile {
    filename: String,
    testcases: Vec<TestCase>,
}

#[test]
fn test_all() {
    let test_folder = "tests";

    let contents_filename = format!("{}/test_contents.json", test_folder);
    let json_data = fs::read_to_string(contents_filename).expect("Unable to read file");

    let test_files: Vec<TestFile> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

    for testfile in test_files {
        let filename = format!("{}/{}", test_folder, testfile.filename);
        compile::compile(&filename, "test_output");
        for testcase in testfile.testcases {
            let binary_status = Command::new("./test_output.out")
                .status()
                .expect("Failed to run the compiled binary");
            assert_eq!(binary_status.code().expect("binary was terminated by OS signal?"), testcase.output_code);
        }
    }
}