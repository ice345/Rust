mod cleanup;
mod editor;
mod cursor;
mod output;
mod reader;
mod editor_rows;
mod constants;
mod editor_contents;

use editor::Editor;
use crossterm::terminal;

fn main() -> crossterm::Result<()> {
    let _clean = cleanup::CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}