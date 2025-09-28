/**
 * tests how my compiler behaves when linked to code compiled with GCC
 */
use std::{fs, io::Write, path::PathBuf, process::{Command, Stdio}, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::compile;

#[derive(Serialize, Deserialize, Debug)]
struct GccLinkedTest {
    foldername: String,
    args: Option<Vec<String>>,
    stdin:Option<String>,
    stdout: Option<String>,
}

#[test]
fn test_linked_with_gcc() {
    let all_tests_folder = PathBuf::from_str("tests/link_with_gcc").unwrap();


    let contents_filename = all_tests_folder.join("test_contents.json");
    let json_data = fs::read_to_string(contents_filename).expect("Unable to read file");

    let test_cases: Vec<GccLinkedTest> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

    for test_case in test_cases {
        let test_folder = all_tests_folder.join(test_case.foldername.clone());//go in the folder that holds the test case
        //compile gcc's part of the test
        let gcc_code = test_folder.join("gcc_code.c");
        let gcc_object = test_folder.join("gcc_code.o");
        let gcc_status = Command::new("gcc")
            .arg(&gcc_code)
            .arg("-o")
            .arg(&gcc_object)
            .arg("-c")//just compile, don't link
            .status().unwrap();

        assert!(gcc_status.success());

        //compile my part of the test
        let filename = test_folder.join("main.c");
        let output_filename = test_folder.join("test_output.out");
        compile::compile(&filename, &output_filename, &[&gcc_object], true, None).unwrap();//compile, and link with gcc's code

        //test the code

        let fixed_args  = test_case.args.or(Some(Vec::new())).unwrap();

        let binary_process = Command::new(output_filename)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(fixed_args)
            .spawn()
            .expect("Failed to run the compiled binary");

        if let Some(text_input) = test_case.stdin {
            //test case requires stdin to be passed
            binary_process.stdin
            .as_ref()
            .unwrap()
            .write_all(text_input.as_bytes())
            .unwrap();
        }

        let binary_command = binary_process
            .wait_with_output()
            .expect("Failed to run test case");

        println!("testing results for {}", test_case.foldername);

        assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), 0);

        if let Some(text_output) = test_case.stdout {
            assert_eq!(String::from_utf8_lossy(&binary_command.stdout), text_output);
        }
    }
}