use std::path::PathBuf;
use vim_editor::editor_rows::EditorRows;

mod common;

#[test]
fn test_open_and_save_file() {
    let test_file = PathBuf::from("test_open_and_save_file.txt");
    let initial_content = "Hello, World!\nThis is a test file.";
    common::setup_test_file(&test_file, initial_content);

    let mut editor_rows = EditorRows::from_file(test_file.clone());
    assert_eq!(editor_rows.number_of_rows(), 2);
    assert_eq!(editor_rows.get_row(0), "Hello, World!");

    editor_rows.insert_char(0, 5, ',');
    editor_rows.save_file().unwrap();

    let updated_content = common::read_test_file(&test_file);
    assert_eq!(updated_content.trim(), "Hello,, World!\nThis is a test file.");

    common::cleanup_test_file(&test_file);
}

