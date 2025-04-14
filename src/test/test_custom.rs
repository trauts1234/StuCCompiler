/**
 * runs feature-specific custom tests based on currently implemented features
 */

use std::{fs, io::Write, path::PathBuf, process::{Command, Stdio}, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::compile;

#[derive(Serialize, Deserialize, Debug)]
struct TestFile {
    filename: String,
    args: Option<Vec<String>>,
    stdin:Option<String>,
    stdout: Option<String>,
    return_code: Option<i32>,
}

#[test]
fn test_all() {
    let test_folder = PathBuf::from_str("tests/standalone").unwrap();

    let contents_filename = test_folder.join("test_contents.json");
    let json_data = fs::read_to_string(contents_filename).expect("Unable to read file");

    let test_cases: Vec<TestFile> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

    for testfile in test_cases {
        let filename = test_folder.join(testfile.filename.clone());
        let output_filename = test_folder.join("test_output.out");
        compile::compile(&filename, &output_filename, &[]).unwrap();

        let fixed_args  = testfile.args.or(Some(Vec::new())).unwrap();

        let binary_process = Command::new(output_filename)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .args(fixed_args)
            .spawn()
            .expect("Failed to run the compiled binary");

        if let Some(text_input) = testfile.stdin {
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

        println!("testing results for {}", testfile.filename);

        if let Some(ret_code) = testfile.return_code {
            assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), ret_code);
        }
        if let Some(text_output) = testfile.stdout {
            assert_eq!(String::from_utf8_lossy(&binary_command.stdout), text_output);
        }
    }
}
