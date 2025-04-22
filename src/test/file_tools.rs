use std::{fs, path::{Path, PathBuf}};


pub fn find_folders<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut folders = Vec::new();

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            folders.push(path);
        }
    }

    folders
}

pub fn find_c_files<P: AsRef<Path>>(dir: P) -> Vec<std::path::PathBuf> {
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