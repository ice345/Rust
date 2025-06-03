use crate::{
    constants::Mode, cursor::CursorController, editor_contents::EditorContents,
    editor_rows::EditorRows,
};
use crossterm::{cursor, execute, queue, style, terminal};
use std::cmp;
use std::io::{Write, stdout};

pub struct Output {
    pub win_size: (usize, usize),
    pub editor_contents: EditorContents,
    pub editor_rows: EditorRows,
    pub cursor_controller: CursorController,
}

impl Output {
    pub fn new() -> Self {
        let win_size = terminal::size()
            .map(|(x, y)| (x as usize, y as usize - 1))
            .unwrap(); // terminal::size() return Result<(u16: column, u16: row)> 类型
        Self {
            win_size,
            editor_contents: EditorContents::new(),
            editor_rows: EditorRows::new(),
            cursor_controller: CursorController::new(win_size),
        }
    }

    fn draw_welcome(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;

        let name_lines: Vec<&str> = crate::constants::NAME.lines().collect();
        let name_height = name_lines.len();
        let vertical_padding = (screen_rows.saturating_sub(name_height)) / 2;

        for i in 0..screen_rows {
            if i < vertical_padding || i >= vertical_padding + name_height {
                self.editor_contents.push('~');
            } else {
                let line = name_lines[i - vertical_padding];
                let line_padding = (screen_columns.saturating_sub(line.len())) / 2;
                (0..line_padding).for_each(|_| self.editor_contents.push(' '));
                self.editor_contents.push_str(line);
            }
            queue!(
                self.editor_contents,
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )
            .unwrap();
            self.editor_contents.push_str("\r\n");
        }
    }

    // fn draw_contents(&mut self) {
    //     let screen_rows = self.win_size.1;
    //     let screen_columns = self.win_size.0;
    //     for i in 0..screen_rows {
    //         let file_row = i + self.cursor_controller.row_offest;
    //         if file_row >= self.editor_rows.number_of_rows() {
    //             self.editor_contents.push('~');
    //         } else {
    //             let row = self.editor_rows.get_row(file_row);
    //             let len = cmp::min(row.len(), screen_columns);
    //             self.editor_contents.push_str(&row[..len]);
    //         }
    //         queue!(
    //             self.editor_contents,
    //             terminal::Clear(terminal::ClearType::UntilNewLine)
    //         )
    //         .unwrap();
    //         self.editor_contents.push_str("\r\n");
    //     }
    // }

    fn draw_contents(&mut self) {
        let screen_rows = self.win_size.1;
        let screen_columns = self.win_size.0;
        for i in 0..screen_rows {
            let file_row = i + self.cursor_controller.row_offest; // row_offest 为一个偏移量(使得文件内容随着光标偏移)
            if file_row >= self.editor_rows.number_of_rows() {
                self.editor_contents.push('~');
            } else {
                let row = self.editor_rows.get_row(file_row);
                if row.is_empty() {
                    // 处理空行的情况
                    // 不需要添加内容
                } else {
                    // 应用水平偏移量
                    let column_offset = self.cursor_controller.column_offest;
                    let start = if column_offset < row.len() {
                        column_offset
                    } else {
                        0
                    }; //判断条件是判断column_offest是否已经使得行内容被偏移到已经看不到
                    let end = row.len();

                    if start < end {
                        let adjusted_row = &row[start..end];
                        let display_length = cmp::min(adjusted_row.len(), screen_columns); // 限制屏幕内显示行的长度

                        // 检查当前行是否有搜索匹配项,高亮显示
                        let matches_in_line: Vec<_> = self
                            .editor_rows
                            .search_matches
                            .iter()
                            .filter(|&&(row, col, _)| {
                                row == file_row && col >= start && col < start + display_length
                            })
                            .collect();

                        if matches_in_line.is_empty() {
                            // 没有匹配项, 正常显示
                            self.editor_contents
                                .push_str(&adjusted_row[..display_length]);
                        } else {
                            // 有匹配项, 高亮显示
                            let mut last_pos = 0;
                            for &(_, col, len) in &matches_in_line {
                                let rel_col = col.saturating_sub(start); // 相对于当前显示窗口的列位置

                                // 先显示匹配前的正常文本
                                if rel_col > last_pos {
                                    // 确保不越界
                                    let end_pos = std::cmp::min(rel_col, adjusted_row.len());
                                    if last_pos < end_pos {
                                        self.editor_contents
                                            .push_str(&adjusted_row[last_pos..end_pos]);
                                    }
                                }

                                // 高亮显示匹配部分
                                let match_end = cmp::min(rel_col + len, display_length);

                                if rel_col < match_end && rel_col < adjusted_row.len() {
                                    let actual_end = std::cmp::min(match_end, adjusted_row.len());

                                    self.editor_contents
                                        .push_str(&style::Attribute::Underlined.to_string());
                                    self.editor_contents
                                        .push_str(&adjusted_row[rel_col..actual_end]);
                                    self.editor_contents
                                        .push_str(&style::Attribute::Reset.to_string());
                                }

                                // self.editor_contents.push_str(&style::Attribute::Underlined.to_string());
                                // self.editor_contents.push_str(&adjusted_row[rel_col..match_end]);
                                // self.editor_contents.push_str(&style::Attribute::Reset.to_string());

                                last_pos = match_end;
                            }

                            // 显示匹配后的剩余文本
                            if last_pos < display_length {
                                self.editor_contents
                                    .push_str(&adjusted_row[last_pos..display_length]);
                            }
                        }
                    }
                }
            }
            queue!(
                self.editor_contents,
                terminal::Clear(terminal::ClearType::UntilNewLine)
            )
            .unwrap();
            self.editor_contents.push_str("\r\n");
        }
    }

    pub fn draw_status_bar(&mut self, mode: &Mode) {
        self.editor_contents
            .push_str(&style::Attribute::Reverse.to_string());
        let info = format!(
            "{} -- {} lines",
            self.editor_rows
                .filename
                .as_ref()
                .and_then(|path| path.file_name())
                .and_then(|name| name.to_str())
                .unwrap_or("[No Name]"),
            self.editor_rows.number_of_rows()
        );

        let mode_str = match mode {
            Mode::Normal => "NORMAL",
            Mode::Command => "COMMAND",
            Mode::Search => "SEARCH",
            Mode::Insert => "INSERT",
        };

        let mode_info = format!(" - {} - ", mode_str);
        let info_len = cmp::min(info.len(), self.win_size.0);
        let line_info = format!(
            "{}/{}",
            self.cursor_controller.cursor_y + 1,
            self.editor_rows.number_of_rows()
        );

        let total_len = info_len + mode_info.len() + line_info.len();
        let padding = if total_len < self.win_size.0 {
            (self.win_size.0 - total_len) / 2
        } else {
            0
        };

        self.editor_contents.push_str(&info[..info_len]);
        for _ in 0..padding {
            self.editor_contents.push(' ');
        }
        self.editor_contents.push_str(&mode_info);
        for _ in 0..padding {
            self.editor_contents.push(' ');
        }
        self.editor_contents.push_str(&line_info);

        self.editor_contents
            .push_str(&style::Attribute::Reset.to_string());
    }

    pub fn draw_rows(&mut self) {
        if self.editor_rows.number_of_rows() == 0 {
            self.draw_welcome();
        } else {
            self.draw_contents();
        }
    }

    pub fn refresh_screen(&mut self, mode: &Mode, command_buffer: &str) -> crossterm::Result<()> {
        self.cursor_controller.scroll();
        queue!(self.editor_contents, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows();
        let status_line_y = self.win_size.1;
        queue!(
            self.editor_contents,
            cursor::MoveTo(0, status_line_y as u16),
            terminal::Clear(terminal::ClearType::UntilNewLine)
        )?;
        self.draw_status_bar(mode);
        // if let Mode::Command = mode {
        //     queue!(
        //         self.editor_contents,
        //         cursor::MoveTo(0, (status_line_y + 1) as u16),
        //         terminal::Clear(terminal::ClearType::UntilNewLine),
        //         style::Print(":"),
        //         style::Print(command_buffer)
        //     )?;
        // }
        if *mode == Mode::Command || *mode == Mode::Search {
            queue!(
                self.editor_contents,
                cursor::MoveTo(0, (status_line_y + 1) as u16),
                terminal::Clear(terminal::ClearType::UntilNewLine),
                style::Print(":"),
                style::Print(command_buffer)
            )?;
        }

        let cursor_y = self
            .cursor_controller
            .cursor_y
            .saturating_sub(self.cursor_controller.row_offest);
        let cursor_x = self
            .cursor_controller
            .cursor_x
            .saturating_sub(self.cursor_controller.column_offest);

        // 添加额外检查确保不会溢出u16
        let cursor_x = std::cmp::min(cursor_x, u16::MAX as usize) as u16;
        let cursor_y = std::cmp::min(cursor_y, u16::MAX as usize) as u16;

        queue!(
            self.editor_contents,
            cursor::MoveTo(cursor_x, cursor_y),
            cursor::Show,
        )?;
        self.editor_contents.flush()
    }
    pub fn clear_screen() -> crossterm::Result<()> {
        execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))
    }

    pub fn move_cursor(&mut self, direction: char, number_of_rows: usize) {
        match direction {
            'h' => {
                if self.cursor_controller.cursor_x > 0 {
                    self.cursor_controller.cursor_x -= 1;
                }
            }
            'j' => {
                if self.cursor_controller.cursor_y < number_of_rows.saturating_sub(1) {
                    self.cursor_controller.cursor_y += 1;
                }
            }
            'k' => {
                if self.cursor_controller.cursor_y > 0 {
                    self.cursor_controller.cursor_y -= 1;
                }
            }
            'l' => {
                // 允许光标在文件内容的情况下根据行长度限制
                if self.editor_rows.number_of_rows() > 0
                    && self.cursor_controller.cursor_y < self.editor_rows.number_of_rows()
                {
                    let row_len = self
                        .editor_rows
                        .get_row(self.cursor_controller.cursor_y)
                        .len();
                    if self.cursor_controller.cursor_x < row_len {
                        self.cursor_controller.cursor_x += 1;
                    }
                } else {
                    // 在欢迎屏幕时，限制在屏幕宽度内
                    if self.cursor_controller.cursor_x < self.win_size.0.saturating_sub(1) {
                        self.cursor_controller.cursor_x += 1;
                    }
                }
            }
            '$' => {
                // 移动到行的实际末尾
                if self.editor_rows.number_of_rows() > 0
                    && self.cursor_controller.cursor_y < self.editor_rows.number_of_rows()
                {
                    let row_len = self
                        .editor_rows
                        .get_row(self.cursor_controller.cursor_y)
                        .len();
                    // 检查行长度，避免在空行上出现问题
                    if row_len > 0 {
                        self.cursor_controller.cursor_x = row_len - 1; // 移动到行的最后一个字符
                    } else {
                        self.cursor_controller.cursor_x = 0; // 空行则移动到行首
                    }
                } else {
                    // 在欢迎屏幕时，限制在屏幕宽度内
                    if self.win_size.0 > 0 {
                        self.cursor_controller.cursor_x = self.win_size.0.saturating_sub(1);
                    }
                }
            }
            '0' => {
                self.cursor_controller.cursor_x = 0;
            }
            _ => {}
        }
    }
}
