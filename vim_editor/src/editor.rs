use crate::{constants::Mode, output::Output, reader::Reader};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub struct Editor {
    reader: Reader,
    output: Output,
    mode: Mode,
    command_buffer: String,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
        }
    }

    pub fn process_keypress(&mut self) -> crossterm::Result<bool> {
        match self.mode {
            Mode::Normal => {
                match self.reader.read_key()? {
                    KeyEvent {
                        code: KeyCode::Char(':'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.mode = Mode::Command;
                        self.command_buffer.clear();
                    }
                    KeyEvent {
                        code: KeyCode::Char('/'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.mode = Mode::Search;
                        self.command_buffer.clear();
                    }
                    KeyEvent {
                        code: KeyCode::Char('i'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.mode = Mode::Insert;
                    }
                    KeyEvent {
                        code: KeyCode::Char('a'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.output.cursor_controller.cursor_x += 1;
                        self.mode = Mode::Insert;
                    }
                    KeyEvent {
                        code: KeyCode::Char(val @ ('h' | 'j' | 'k' | 'l' | '0' | '$')),
                        modifiers: KeyModifiers::NONE,
                    } =>
                    /* self.output.move_cursor(val, self.output.win_size), */
                    {
                        if self.output.editor_rows.number_of_rows() == 0 {
                            self.output.move_cursor(val, self.output.win_size.0);
                        } else {
                            self.output
                                .move_cursor(val, self.output.editor_rows.number_of_rows());
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Up,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.output
                            .move_cursor('k', self.output.editor_rows.number_of_rows());
                    }
                    KeyEvent {
                        code: KeyCode::Down,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.output
                            .move_cursor('j', self.output.editor_rows.number_of_rows());
                    }
                    KeyEvent {
                        code: KeyCode::Left,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.output
                            .move_cursor('h', self.output.editor_rows.number_of_rows());
                    }
                    KeyEvent {
                        code: KeyCode::Right,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.output
                            .move_cursor('l', self.output.editor_rows.number_of_rows());
                    }
                    KeyEvent {
                        code: KeyCode::Char('n'),
                        modifiers: KeyModifiers::NONE,
                    } => {
                        // 搜索下一个匹配项
                        if let Some((row, col)) = self.output.editor_rows.next_match(
                            self.output.cursor_controller.cursor_y,
                            self.output.cursor_controller.cursor_x,
                        ) {
                            self.output.cursor_controller.cursor_y = row;
                            self.output.cursor_controller.cursor_x = col;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('N'),
                        modifiers: KeyModifiers::SHIFT,
                    } => {
                        // 搜索下一个匹配项
                        if let Some((row, col)) = self.output.editor_rows.prev_match(
                            self.output.cursor_controller.cursor_y,
                            self.output.cursor_controller.cursor_x,
                        ) {
                            self.output.cursor_controller.cursor_y = row;
                            self.output.cursor_controller.cursor_x = col;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Char('q'),
                        modifiers: KeyModifiers::CONTROL,
                    } => return Ok(false),
                    _ => {}
                }
            }
            Mode::Command => match self.reader.read_key()? {
                KeyEvent {
                    code: KeyCode::Char(ch),
                    modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                } => {
                    self.command_buffer.push(ch);
                }
                KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if self.command_buffer == "q" {
                        return Ok(false);
                    }
                    if self.command_buffer == "gg" {
                        self.output.cursor_controller.cursor_x = 0;
                        self.output.cursor_controller.cursor_y = 0;
                    }
                    if self.command_buffer == "G" {
                        self.output.cursor_controller.cursor_y =
                            if self.output.editor_rows.number_of_rows() == 0 {
                                self.output.win_size.1.saturating_sub(1)
                            } else {
                                self.output.editor_rows.number_of_rows().saturating_sub(1)
                            };
                        self.output.cursor_controller.cursor_x = 0;
                    }
                    if self.command_buffer.parse::<usize>().is_ok() {
                        let line = self.command_buffer.parse::<usize>().unwrap();
                        self.output.cursor_controller.cursor_y =
                            if line != 0 && line <= self.output.editor_rows.number_of_rows() {
                                line.saturating_sub(1)
                            } else {
                                0
                            };
                        self.output.cursor_controller.cursor_x = 0;
                    }
                    if self.command_buffer == "w" {
                        match self.output.editor_rows.save_file() {
                            Ok(_) => {
                                self.command_buffer.clear();
                                self.mode = Mode::Normal;
                            }
                            Err(e) => {
                                self.command_buffer = format!("Error: {}", e);
                                self.mode = Mode::Normal;
                            }
                        }
                        self.command_buffer.clear();
                        self.mode = Mode::Normal;
                    }
                    if self.command_buffer == "wq" {
                        match self.output.editor_rows.save_file() {
                            Ok(_) => {
                                self.command_buffer.clear();
                                return Ok(false);
                            }
                            Err(e) => {
                                self.command_buffer = format!("Error: {}", e);
                                self.mode = Mode::Normal;
                            }
                        }
                        self.command_buffer.clear();
                        self.mode = Mode::Normal;
                    }
                    if self.command_buffer == "q!" {
                        self.command_buffer.clear();
                        return Ok(false);
                    }
                    if self.command_buffer == "dd" {
                        self.output
                            .editor_rows
                            .delete_line(self.output.cursor_controller.cursor_y);
                    }

                    self.command_buffer.clear();
                    self.mode = Mode::Normal;
                }
                KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                } => {
                    if !self.command_buffer.is_empty() {
                        self.command_buffer.pop();
                    }
                }
                KeyEvent {
                    code: KeyCode::Esc,
                    modifiers: KeyModifiers::NONE,
                } => {
                    self.command_buffer.clear();
                    self.mode = Mode::Normal;
                }
                _ => {}
            },
            Mode::Search => {
                match self.reader.read_key()? {
                    KeyEvent {
                        code: KeyCode::Char(ch),
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    } => {
                        self.command_buffer.push(ch);

                        // 实时搜索:每输入一个字符就更新搜索
                        if let Some((row, col)) =
                            self.output.editor_rows.search(&self.command_buffer)
                        {
                            // 光标跳到第一个匹配项
                            self.output.cursor_controller.cursor_y = row;
                            self.output.cursor_controller.cursor_x = col;
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        // 确认搜索, 保留高亮度并返回普通模式
                        self.mode = Mode::Normal;
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        if !self.command_buffer.is_empty() {
                            self.command_buffer.pop();
                            // 更新搜索结果
                            if self.command_buffer.is_empty() {
                                self.output.editor_rows.search_term = None;
                                self.output.editor_rows.search_matches.clear();
                            } else if let Some((row, col)) =
                                self.output.editor_rows.search(&self.command_buffer)
                            {
                                // 光标跳到第一个匹配项
                                self.output.cursor_controller.cursor_y = row;
                                self.output.cursor_controller.cursor_x = col;
                            }
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        self.command_buffer.clear();
                        self.output.editor_rows.search_term = None;
                        self.output.editor_rows.search_matches.clear();
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
            }
            Mode::Insert => {
                match self.reader.read_key()? {
                    KeyEvent {
                        code: KeyCode::Char(ch),
                        modifiers: KeyModifiers::NONE | KeyModifiers::SHIFT,
                    } => {
                        // 在光标位置插入字符
                        self.output.editor_rows.insert_char(
                            self.output.cursor_controller.cursor_y,
                            self.output.cursor_controller.cursor_x,
                            ch,
                        );
                        // 光标右移
                        self.output.cursor_controller.cursor_x += 1;
                    }
                    KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        // 插入新行
                        self.output.editor_rows.insert_newline(
                            self.output.cursor_controller.cursor_y,
                            self.output.cursor_controller.cursor_x,
                        );
                        // 光标移动到下一行开始
                        self.output.cursor_controller.cursor_y += 1;
                        self.output.cursor_controller.cursor_x = 0;
                    }
                    KeyEvent {
                        code: KeyCode::Backspace,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        if self.output.cursor_controller.cursor_x > 0 {
                            // 删除光标前的字符
                            self.output.cursor_controller.cursor_x -= 1;
                            self.output.editor_rows.delete_char(
                                self.output.cursor_controller.cursor_y,
                                self.output.cursor_controller.cursor_x,
                            );
                        } else if self.output.cursor_controller.cursor_y > 0 {
                            // 在行首删除，需要将光标移到上一行末尾
                            let prev_row_len = self
                                .output
                                .editor_rows
                                .get_row(self.output.cursor_controller.cursor_y - 1)
                                .len();
                            self.output.cursor_controller.cursor_y -= 1;
                            self.output.cursor_controller.cursor_x = prev_row_len;
                            // 合并行
                            self.output.editor_rows.delete_char(
                                self.output.cursor_controller.cursor_y,
                                self.output.cursor_controller.cursor_x,
                            );
                        }
                    }
                    KeyEvent {
                        code: KeyCode::Delete,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        // 删除光标处的字符
                        self.output.editor_rows.delete_char(
                            self.output.cursor_controller.cursor_y,
                            self.output.cursor_controller.cursor_x,
                        );
                    }
                    KeyEvent {
                        code: KeyCode::Esc,
                        modifiers: KeyModifiers::NONE,
                    } => {
                        // 返回普通模式
                        self.mode = Mode::Normal;
                    }
                    _ => {}
                }
            }
        }
        Ok(true)
    }

    pub fn run(&mut self) -> crossterm::Result<bool> {
        // 首先刷新屏幕,显示当前状态
        self.output
            .refresh_screen(&self.mode, &self.command_buffer)?;
        // 处理按键输入
        let continue_running = self.process_keypress()?;

        // 在Insert模式下, 立即刷新屏幕以显示更改
        if self.mode == Mode::Insert {
            self.output
                .refresh_screen(&self.mode, &self.command_buffer)?;
        }

        Ok(continue_running)
    }
}
