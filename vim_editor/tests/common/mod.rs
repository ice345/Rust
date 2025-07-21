use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

pub fn setup_test_file(test_file: &Path, content: &str) {
    let mut file = File::create(test_file).unwrap();
    writeln!(file, "{}", content).unwrap();
}

pub fn read_test_file(test_file: &Path) -> String {
    fs::read_to_string(test_file).unwrap()
}

pub fn cleanup_test_file(test_file: &Path) {
    fs::remove_file(test_file).unwrap();
}
