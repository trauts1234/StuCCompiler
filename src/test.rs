#[cfg(test)]
pub mod test {
    use std::{fs, io::Write, process::{Command, Stdio}};

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
        let test_folder = "tests";

        let contents_filename = format!("{}/test_contents.json", test_folder);
        let json_data = fs::read_to_string(contents_filename).expect("Unable to read file");

        let test_cases: Vec<TestFile> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

        for testfile in test_cases {
            let filename = format!("{}/{}", test_folder, testfile.filename);
            let output_filename = format!("{}/{}", test_folder, "test_output");
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

    #[derive(Serialize, Deserialize, Debug)]
    struct GccLinkedTest {
        foldername: String,
        args: Option<Vec<String>>,
        stdin:Option<String>,
        stdout: Option<String>,
        return_code: Option<i32>,
    }

    #[test]
    fn test_linked_with_gcc() {
        let all_tests_folder = "tests/link_with_gcc";

        let contents_filename = format!("{}/test_contents.json", all_tests_folder);
        let json_data = fs::read_to_string(contents_filename).expect("Unable to read file");

        let test_cases: Vec<GccLinkedTest> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

        for test_case in test_cases {
            let test_folder = format!("{}/{}", all_tests_folder, test_case.foldername);//go in the folder that holds the test case
            //compile gcc's part of the test
            let gcc_code = format!("{}/{}", test_folder, "gcc_code.c");
            let gcc_object = format!("{}/{}", test_folder, "gcc_code.o");
            let gcc_status = Command::new("gcc")
                .arg(&gcc_code)
                .arg("-o")
                .arg(&gcc_object)
                .arg("-c")//just compile, don't link
                .status().unwrap();

            assert!(gcc_status.success());

            //compile my part of the test
            let filename = format!("{}/{}", test_folder, "main.c");
            let output_filename = format!("{}/{}", test_folder, "test_output");
            compile::compile(&filename, &output_filename, &[&gcc_object]).unwrap();//compile, and link with gcc's code

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

            if let Some(ret_code) = test_case.return_code {
                assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), ret_code);
            }
            if let Some(text_output) = test_case.stdout {
                assert_eq!(String::from_utf8_lossy(&binary_command.stdout), text_output);
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug)]
    struct MultiFileTest {
        filename: String,
        return_code: i32,
    }

    /*#[test]
    fn test_multifile() {
        todo!()
    }*/
}