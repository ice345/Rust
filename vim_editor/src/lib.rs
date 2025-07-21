pub mod cleanup;
pub mod constants;
pub mod cursor;
pub mod editor;
pub mod editor_contents;
pub mod editor_rows;
pub mod output;
pub mod reader;

use crossterm::terminal;
use editor::Editor;

pub fn run() -> crossterm::Result<()> {
    let _clean = cleanup::CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}