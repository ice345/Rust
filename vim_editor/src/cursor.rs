pub struct CursorController {
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub screen_columns: usize,
    pub screen_rows: usize,
    pub row_offest: usize,
    pub column_offest: usize,
}

impl CursorController {
    /// `win_size` is a tuple of (column, row)
    pub fn new(win_size: (usize, usize)) -> Self {
        Self {
            cursor_x: 0,
            cursor_y: 0,
            screen_columns: win_size.0,
            screen_rows: win_size.1,
            row_offest: 0,
            column_offest: 0,
        }
    }

    pub fn scroll(&mut self) {
        // 垂直滚动
        if self.cursor_y < self.row_offest {
            self.row_offest = self.cursor_y;
        }
        if self.cursor_y >= self.row_offest + self.screen_rows {
            self.row_offest = self.cursor_y - self.screen_rows + 1;
        }

        // 水平滚动
        if self.cursor_x < self.column_offest {
            self.column_offest = self.cursor_x;
        }
        if self.cursor_x >= self.column_offest + self.screen_columns {
            self.column_offest = self.cursor_x - self.screen_columns + 1;
        }
    }
}
