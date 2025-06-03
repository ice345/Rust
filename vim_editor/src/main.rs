mod cleanup;
mod constants;
mod cursor;
mod editor;
mod editor_contents;
mod editor_rows;
mod output;
mod reader;

use crossterm::terminal;
use editor::Editor;

fn main() -> crossterm::Result<()> {
    let _clean = cleanup::CleanUp;
    terminal::enable_raw_mode()?;

    let mut editor = Editor::new();
    while editor.run()? {}

    Ok(())
}
