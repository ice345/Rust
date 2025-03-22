use std::fs;
use std::path::PathBuf;

pub struct EditorRows {
    pub row_contents: Vec<Box<String>>,
    pub filename: Option<PathBuf>,

    pub search_term: Option<String>,
    pub search_matches: Vec<(usize, usize, usize)>, // (行号, 起始列, 长度)
}

impl EditorRows {
    pub fn new() -> Self {
        let mut arg = std::env::args().skip(1);

        match arg.next() {
            None => {
                eprintln!("No file provided.");
                Self {
                    row_contents: Vec::new(),
                    filename: None,
                    search_term: None,
                    search_matches: Vec::new(),
                }
            }
            Some(file) => {
                if let Err(err) = fs::metadata(&file) {
                    eprintln!("Error: Cannot file {}: {}", file, err);
                    Self {
                        row_contents: Vec::new(),
                        filename: None,
                        search_term: None,
                        search_matches: Vec::new(),
                    }
                } else {
                    Self::from_file(file.into())
                }
            }
        }
    }

    pub fn from_file(file: PathBuf) -> Self {
        let file_content = fs::read_to_string(&file).expect("Unable to read file");
        Self {
            filename: Some(file),
            row_contents: file_content
                .lines()
                .map(|it| Box::new(it.to_string()))
                .collect(),
            search_term: None,
            search_matches: Vec::new(),
        }
    }


    pub fn search(&mut self, query: &str) -> Option<(usize, usize)> {
        // self.search_term = if query.is_empty() {None} else { Some(query.to_string()) };
        // 清空之前的搜索结果
        self.search_matches.clear();

        if query.is_empty() || self.row_contents.is_empty() {
            self.search_term = None;
            return None;
        }

        // 查找所有匹配的项并存储
        // for (row_idx, row) in self.row_contents.iter().enumerate() {
        //     let mut col_idx = 0;
        //     while let Some(pos) = row[col_idx..].find(query) {
        //         let match_pos = col_idx + pos;
        //         self.search_matches.push((row_idx, match_pos, query.len()));
        //         col_idx += match_pos + 1; //继续查找下一个匹配
        //     }
        // }

        // 保存当前搜索词
        self.search_term = Some(query.to_string());
        
        // 查找所有匹配项
        for (row_idx, row) in self.row_contents.iter().enumerate() {
            let mut col_idx = 0;
            
            // 安全地查找所有匹配项
            while let Some(pos) = match row[col_idx..].find(query) {
                Some(p) => Some(p),
                None => None, // 处理可能的None值
            } {
                let match_pos = col_idx + pos;
                // 保存匹配项的位置和长度
                self.search_matches.push((row_idx, match_pos, query.len()));
                
                // 防止无限循环，确保col_idx会前进(问题出自这里, 举个例子:如果你跳转到最后一行,只有一个不匹配的字符,就会陷入无限循环)
                if match_pos + 1 <= row.len() {
                    col_idx = match_pos + 1;
                } else {
                    break;
                }
            }
        }

        // 返回第一个匹配项(如果有)
        self.search_matches.first().map(|&(row, col, _)| (row, col))
    }

    pub fn next_match(&self, current_row: usize, current_col: usize) -> Option<(usize, usize)> {
        if self.search_matches.is_empty() {
            return None;
        }

        // 查找当前位置后的下一个匹配项
        for &(row, col, _) in &self.search_matches {
            if row > current_row || (row == current_row && col > current_col) {
                return Some((row, col));
            }
        }

        // 如果没有找到, 则返回第一个匹配项(循环)
        Some((self.search_matches[0].0, self.search_matches[0].1))
    }

    pub fn prev_match(&self, current_row: usize, current_col: usize) -> Option<(usize, usize)> {
        if self.search_matches.is_empty() {
            return None;
        }

        // 查找当前位置前的上一个匹配项
        // let mut query: Option<&str> = None;
        for &(row, col, _) in self.search_matches.iter().rev() {
            if row < current_row || (row == current_row && col < current_col) {
                return Some((row, col));
            }
        }

        // 如果没有找到, 则返回最后一个匹配项(循环)
        let last = self.search_matches.len() - 1;
        Some((self.search_matches[last].0, self.search_matches[last].1))
    }

    // return the line count
    pub fn number_of_rows(&self) -> usize {
        self.row_contents.len()
    }

    // return the row at the given index, otherwise return an empty string reference(if the index is out of bounds)
    pub fn get_row(&self, at: usize) -> &String {
        if at < self.row_contents.len() {
            &self.row_contents[at]
        } else {
            // 返回空字符串引用（使用静态生命周期）
            static EMPTY: String = String::new();
            &EMPTY
        }
    }

    // 在指定位置插入字符
    pub fn insert_char(&mut self, at_row: usize, at_col: usize, ch: char) {
        // 如果行号超出范围，添加新行直到达到要求的行
        while at_row >= self.row_contents.len() {
            self.row_contents.push(Box::new(String::new()));
        }
        
        // 获取指定行并插入字符
        let row = &mut self.row_contents[at_row];
        if at_col > row.len() {
            // 如果列号超出范围，填充空格
            row.push_str(&" ".repeat(at_col - row.len()));
            row.push(ch);
        } else {
            // 否则在指定位置插入
            row.insert(at_col, ch);
        }
    }

    // 在指定位置删除字符
    pub fn delete_char(&mut self, at_row: usize, at_col: usize) -> bool {
        // 检查行是否存在
        if at_row >= self.row_contents.len() {
            return false;
        }
        
        // 直接在原始数据上操作，不要克隆
        if at_col >= self.row_contents[at_row].len() {
            // 在行尾删除，需要与下一行合并
            if at_row < self.row_contents.len() - 1 {
                // 获取下一行内容并移除
                let next_row = self.row_contents.remove(at_row + 1);
                // 将下一行内容追加到当前行
                self.row_contents[at_row].push_str(&next_row);
                return true;
            }
            return false;
        } else {
            // 删除指定位置的字符
            self.row_contents[at_row].remove(at_col);
            return true;
        }
    }

    // 删除指定行
    pub fn delete_line(&mut self, at_row: usize) -> bool {
        // 检查行是否存在
        if at_row >= self.row_contents.len() {
            return false;
        }
        
        // 直接在原始数据上操作，不要克隆
        self.row_contents.remove(at_row);
        return true;
    }

    // 处理回车键，分割行
    pub fn insert_newline(&mut self, at_row: usize, at_col: usize) {
        // 如果行号超出范围，添加新行
        while at_row >= self.row_contents.len() {
            self.row_contents.push(Box::new(String::new()));
        }
        
        // 获取当前行
        let current_row = &mut self.row_contents[at_row];
        
        // 创建新行
        let new_row = if at_col >= current_row.len() {
            // 如果在行尾，创建空行
            Box::new(String::new())
        } else {
            // 否则分割当前行
            let remainder = current_row[at_col..].to_string();
            current_row.truncate(at_col);
            Box::new(remainder)
        };
        
        // 插入新行
        self.row_contents.insert(at_row + 1, new_row);
    }

    // 保存文件
    pub fn save_file(&self) -> std::io::Result<()> {
        match &self.filename {
            Some(path) => {
                // 将所有行连接成一个字符串，使用换行符分隔
                let content = self.row_contents.iter()
                    .map(|row| row.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n");
                
                // 写入文件
                std::fs::write(path, content)
            }
            None => {
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "No filename specified"))
            }
        }
    }

}