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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cursor_controller() {
        let (width, height) = (100, 50);
        let controller = CursorController::new((width, height));
        assert_eq!(controller.cursor_x, 0);
        assert_eq!(controller.cursor_y, 0);
        assert_eq!(controller.screen_columns, width);
        assert_eq!(controller.screen_rows, height);
        assert_eq!(controller.row_offest, 0);
        assert_eq!(controller.column_offest, 0);
    }

    #[test]
    fn test_scroll_vertical() {
        let (width, height) = (100, 10);
        let mut controller = CursorController::new((width, height));

        // Scroll down
        controller.cursor_y = 15;
        controller.scroll();
        assert_eq!(controller.row_offest, 6);

        // Scroll up
        controller.cursor_y = 2;
        controller.scroll();
        assert_eq!(controller.row_offest, 2);
    }

    #[test]
    fn test_scroll_horizontal() {
        let (width, height) = (20, 50);
        let mut controller = CursorController::new((width, height));

        // Scroll right
        controller.cursor_x = 25;
        controller.scroll();
        assert_eq!(controller.column_offest, 6);

        // Scroll left
        controller.cursor_x = 3;
        controller.scroll();
        assert_eq!(controller.column_offest, 3);
    }
}
