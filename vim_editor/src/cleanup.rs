use crossterm::terminal;

pub struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off Raw mode");
        crate::output::Output::clear_screen().expect("error");
    }
}
