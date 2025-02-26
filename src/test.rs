#[cfg(test)]
pub mod test {
    use std::{fs, process::{Command, Stdio}};

    use serde::{Deserialize, Serialize};

    use crate::compile;

    #[derive(Serialize, Deserialize, Debug)]
    struct TestFile {
        filename: String,
        args: Option<Vec<String>>,
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

            let binary_command = Command::new("./test_output.out")
            .stdout(Stdio::piped())
            .args(fixed_args)
            .spawn()
            .and_then(|cmd| cmd.wait_with_output()).expect("Failed to run the compiled binary");

            println!("testing file name: {}", testfile.filename);

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