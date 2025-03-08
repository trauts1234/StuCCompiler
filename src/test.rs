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

        let test_files: Vec<TestFile> = serde_json::from_str(&json_data).expect("Unable to parse JSON");

        for testfile in test_files {
            let filename = format!("{}/{}", test_folder, testfile.filename);
            compile::compile(&filename, "test_output").unwrap();

            let fixed_args  = testfile.args.or(Some(Vec::new())).unwrap();

            let binary_process = Command::new("./test_output")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .args(fixed_args)
                .spawn()
                .expect("Failed to run the compiled binary");

            println!("testing file name: {}", testfile.filename);

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

            if let Some(ret_code) = testfile.return_code {
                assert_eq!(binary_command.status.code().expect("binary was terminated by OS signal?"), ret_code);
            }
            if let Some(text_output) = testfile.stdout {
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