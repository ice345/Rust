use rustyline::completion::FilenameCompleter;
use rustyline::error::ReadlineError;
use rustyline::highlight::{CmdKind, Highlighter, MatchingBracketHighlighter};
use rustyline::hint::HistoryHinter;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Completer, Helper, Hinter, Validator};
use rustyline::{CompletionType, Config, EditMode, Editor, Result};
use std::borrow::Cow::{self, Borrowed, Owned};
use std::collections::HashMap;
use std::env;
use std::fs::OpenOptions;
use std::path::Path;
use std::process::Command;
use std::process::{Child, Stdio};

// ======================= Helper 定義 (保持不變) =======================

#[derive(Helper, Completer, Hinter, Validator)]
struct MyHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    highlighter: MatchingBracketHighlighter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,
    #[rustyline(Hinter)]
    hinter: HistoryHinter,
}

impl Highlighter for MyHelper {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        _default: bool,
    ) -> Cow<'b, str> {
        Borrowed(prompt)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[32m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highlighter.highlight_char(line, pos, kind)
    }
}

// ======================= Tokenizer 重構 =======================

#[derive(Debug, PartialEq)]
enum ParseStatus {
    Complete(Vec<String>), // 解析完成，返回參數列表
    Incomplete(char),      // 未閉合的引號 (' 或 ")
    EscapedAtEnd,          // 行尾有反斜槓，表示續行
    Error(String),         // 語法錯誤
}

/// 重寫後的分詞器
/// 特點：
/// 1. 嚴格區分單雙引號環境
/// 2. 處理行尾反斜槓邏輯
fn tokenize(input: &str) -> ParseStatus {
    let mut tokens = Vec::new();
    let mut current_token = String::new();
    let mut chars = input.chars().peekable();

    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while let Some(c) = chars.next() {
        // 錯誤檢測
        if !in_single_quote && !in_double_quote && c == '|' {
            // 管道符號不能連續出現
            if let Some(&next_char) = chars.peek()
                && next_char == '|'
            {
                return ParseStatus::Error("Syntax error near '||'".to_string());
            }
        }

        if in_single_quote {
            // === 單引號模式 ===
            // 單引號內，除單引號本身外，所有字符（包括 \ " \n）都原樣保留
            if c == '\'' {
                in_single_quote = false;
            } else {
                current_token.push(c);
            }
        } else if in_double_quote {
            // === 雙引號模式 ===
            // 雙引號內支持 \ 轉義
            if c == '\\' {
                if let Some(&next_char) = chars.peek() {
                    // 處理雙引號內的轉義字符
                    match next_char {
                        '"' | '\\' | '$' => {
                            chars.next(); // 吃掉下一個字符
                            current_token.push(next_char);
                        }
                        '\n' => {
                            // 雙引號內的行尾反斜槓：刪除換行符（類似 bash）
                            chars.next();
                        }
                        _ => {
                            // 其他字符，保留反斜槓
                            current_token.push(c);
                        }
                    }
                } else {
                    // 4. 行尾的反斜槓 (Edge Case)
                    // 這是最關鍵的一步：
                    // 如果字符串以 `\` 結尾，且 peek 是 None，說明這行輸入結束了。
                    // 我們必須把這個 `\` 保留下來！
                    // 爲什麼？
                    // 因爲主循環會在後面補一個 `\n`，然後再次調用 tokenize。
                    // 下一次 tokenize 時，這裡的 `\` 就會變成上面的 Case 2 (遇到 \n)，
                    // 從而觸發消除邏輯。
                    current_token.push('\\');
                }
            } else if c == '"' {
                in_double_quote = false;
            } else {
                current_token.push(c);
            }
        } else {
            // === 普通模式 ===
            match c {
                '\'' => in_single_quote = true,
                '"' => in_double_quote = true,
                ' ' | '\t' | '\n' => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                '\\' => {
                    // 普通模式下的反斜槓
                    if let Some(&next_char) = chars.peek() {
                        if next_char == '\n' {
                            // 行尾反斜槓 -> 這是續行符，跳過換行，不生成 token
                            chars.next();
                            // 注意：這裡應該返回 EscapedAtEnd，但我們需要遍歷完才知道是不是最後
                            // 這裡我們暫時不做處理，依賴外部循環邏輯，或者如果這是最後一個字符
                        } else {
                            // 轉義下一個字符（變成普通字符）
                            chars.next();
                            current_token.push(next_char);
                        }
                    } else {
                        // 整個輸入以 \ 結尾
                        return ParseStatus::EscapedAtEnd;
                    }
                }
                _ => current_token.push(c),
            }
        }
    }

    // 檢查結束狀態
    if in_single_quote {
        return ParseStatus::Incomplete('\'');
    }
    if in_double_quote {
        return ParseStatus::Incomplete('"');
    }

    if !current_token.is_empty() {
        tokens.push(current_token);
    }

    // 特別檢查：如果輸入以反斜槓結尾（且不在引號內），上面的 loop 會在 peek 失敗時返回 EscapedAtEnd
    // 但如果是有內容的行，我們在 loop 內部已經處理了。
    // 這裡我們需要一個更簡單的方法檢測行尾反斜槓：
    // 如果 input trim 後以 \ 結尾，且不在引號內。
    // 但爲了準確，我們依賴上面的邏輯。

    // 修正：上面的 loop 邏輯中，如果 input 是 "echo hello \"，最後讀到 '\'，peek 為 None
    // 會進入 return ParseStatus::EscapedAtEnd

    ParseStatus::Complete(tokens)
}

// ======================= 執行邏輯 (變量展開 & 重定向) =======================

fn expand_variables(raw_args: Vec<String>) -> Vec<String> {
    raw_args
        .into_iter()
        .map(|arg| {
            if arg.starts_with('$') && arg.len() > 1 {
                let var_name = &arg[1..];
                env::var(var_name).unwrap_or_else(|_| "".to_string())
            } else {
                arg
            }
        })
        .collect()
}

fn parse_redirection(args: Vec<String>) -> (Vec<String>, Option<Stdio>, Option<Stdio>) {
    let mut clean_args = Vec::new();
    let mut stdin_redirection = None;
    let mut stdout_redirection = None;
    let mut iter = args.into_iter();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            ">" | ">>" => {
                let is_append = arg == ">>";
                if let Some(filename) = iter.next() {
                    let file = OpenOptions::new()
                        .create(true)
                        .append(is_append)
                        .write(true)
                        .truncate(!is_append) // > 應該覆蓋
                        .open(&filename);
                    match file {
                        Ok(f) => stdout_redirection = Some(Stdio::from(f)),
                        Err(e) => eprintln!("Error opening file {}: {}", filename, e),
                    }
                } else {
                    eprintln!("Missing filename for redirection");
                }
            }
            "<" => {
                if let Some(filename) = iter.next() {
                    match OpenOptions::new().read(true).open(&filename) {
                        Ok(f) => stdin_redirection = Some(Stdio::from(f)),
                        Err(e) => eprintln!("Error opening file {}: {}", filename, e),
                    }
                }
            }
            _ => clean_args.push(arg),
        }
    }
    (clean_args, stdin_redirection, stdout_redirection)
}

fn execute_pipeline(input: Vec<String>) {
    // 這裡我們需要重新把 token 列表組裝起來處理管道
    // 但爲了簡單，我們這裡假設 input 已經被切分好了，或者我們在外部處理管道。
    // **注意**：原本的 execute 是傳入 &str 再 split('|')。
    // 這樣做在引號內有 | 時會有 bug (例如: echo "a|b")。
    // 最好的做法是先 tokenize，再根據 token 中的 "|" 符號進行分割。

    // 簡單起見，我們這裡先用一個簡化的邏輯：
    // 如果 token 中包含 "|" 字符串，就分割。

    let mut commands = Vec::new();
    let mut current_cmd_args = Vec::new();

    for token in input {
        if token == "|" {
            // 当前面没有命令却又有|，报错
            if current_cmd_args.is_empty() {
                eprintln!("anishell: syntax error near '|'");
                return;
            }
            commands.push(current_cmd_args);
            current_cmd_args = Vec::new();
        } else {
            current_cmd_args.push(token);
        }
    }
    // 推入最後一個命令
    if !current_cmd_args.is_empty() {
        if !commands.is_empty() && current_cmd_args.is_empty() {
            eprintln!("anishell: syntax error near '|'");
            return;
        } else {
            commands.push(current_cmd_args);
        }
    }

    // 管道執行邏輯
    let mut previous_command: Option<Child> = None;
    let mut commands_iter = commands.into_iter().peekable();

    while let Some(mut args) = commands_iter.next() {
        // 1. 變量展開
        args = expand_variables(args);

        // 2. 提取命令
        if args.is_empty() {
            continue;
        }
        let command = args.remove(0); // 取出第一個作爲命令

        // 3. 處理重定向
        let (final_args, file_stdin, file_stdout) = parse_redirection(args);

        // 4. 設置 Stdin
        let stdin = if let Some(mut previous) = previous_command {
            let output = previous.stdout.take().expect("Failed to take stdout");
            Stdio::from(output)
        } else if let Some(f) = file_stdin {
            f
        } else {
            Stdio::inherit()
        };

        // 5. 設置 Stdout
        let stdout = if commands_iter.peek().is_some() {
            Stdio::piped()
        } else if let Some(f) = file_stdout {
            f
        } else {
            Stdio::inherit()
        };

        // 6. 執行
        match Command::new(&command)
            .args(final_args)
            .stdin(stdin)
            .stdout(stdout)
            .spawn()
        {
            Ok(child) => previous_command = Some(child),
            Err(e) => {
                eprintln!("anishell: command not found: {} ({})", command, e);
                previous_command = None; // 管道斷裂
                break;
            }
        }
    }

    if let Some(mut final_child) = previous_command {
        let _ = final_child.wait();
    }
}

// ======================= Main Loop =======================

fn main() -> Result<()> {
    println!("Welcome to AniShell! (type 'exit' to exit)");

    let config = Config::builder()
        .history_ignore_space(true)
        .history_ignore_dups(true)?
        .completion_type(CompletionType::List)
        // .completion_type(CompletionType::Circular)
        .edit_mode(EditMode::Vi)
        .build();

    let helper = MyHelper {
        completer: FilenameCompleter::new(),
        highlighter: MatchingBracketHighlighter::new(),
        validator: MatchingBracketValidator::new(),
        hinter: HistoryHinter {},
    };

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(helper));

    // 強制加載歷史
    let history_file = "history.txt";
    if rl.load_history(history_file).is_err() {
        println!("No previous history.");
    }

    let mut aliases: HashMap<String, String> = HashMap::new();
    let mut command_buffer = String::new();

    loop {
        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new("/").to_path_buf());

        // 動態 Prompt
        let prompt = if command_buffer.is_empty() {
            format!("{}> ", current_dir.display())
        } else {
            "quote> ".to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let line_trim = line.trim_end(); // 去掉輸入末尾的 \n (如果有)

                if command_buffer.is_empty() {
                    command_buffer.push_str(line_trim);
                } else {
                    // 如果是多行輸入，手動補回換行符
                    command_buffer.push('\n');
                    command_buffer.push_str(line_trim);
                }

                // 嘗試解析
                match tokenize(&command_buffer) {
                    ParseStatus::Complete(args) => {
                        // === 解析成功 ===

                        // 1. 添加到歷史 (整塊添加)
                        rl.add_history_entry(command_buffer.as_str())?;

                        // 2. 處理 Alias (簡單處理第一個詞)
                        let mut final_args = args;
                        if !final_args.is_empty()
                            && let Some(real_cmd) = aliases.get(&final_args[0])
                        {
                            // 這裡簡單替換，實際可能需要再次 tokenize alias 的值
                            final_args[0] = real_cmd.clone();
                        }

                        // 3. 內建命令與執行
                        if !final_args.is_empty() {
                            match final_args[0].as_str() {
                                "exit" => break,
                                "alias" => {
                                    // 簡單的 alias 實現
                                    if final_args.len() > 1 {
                                        // 這裡只是演示，實際上 alias 解析會比較複雜
                                        let parts: Vec<&str> =
                                            final_args[1].splitn(2, '=').collect();
                                        if parts.len() == 2 {
                                            aliases
                                                .insert(parts[0].to_string(), parts[1].to_string());
                                            println!("Alias set: {} -> {}", parts[0], parts[1]);
                                        }
                                    } else {
                                        for (k, v) in &aliases {
                                            println!("{}={}", k, v);
                                        }
                                    }
                                }
                                "cd" => {
                                    let tartget_dir = if final_args.len() > 1 {
                                        Path::new(&final_args[1]).to_path_buf()
                                    } else {
                                        // 用戶只輸入 cd，切換到 home 目錄
                                        match dirs::home_dir() {
                                            Some(path) => path,
                                            None => {
                                                eprintln!("anishell: cd: HOME not set");
                                                continue;
                                            }
                                        }
                                    };

                                    // 嘗試切換目錄
                                    if let Err(e) = env::set_current_dir(&tartget_dir) {
                                        eprintln!("anishell: cd: {}: {}", tartget_dir.display(), e);
                                    }
                                }
                                _ => {
                                    // 執行外部命令
                                    // 這裡傳入的是 Vec<String>，所以 execute_pipeline 改爲接收這個
                                    execute_pipeline(final_args);
                                }
                            }
                        }

                        // 清空 buffer 準備下一次輸入
                        command_buffer.clear();
                    }
                    ParseStatus::Incomplete(_) => {
                        // 繼續循環，顯示 quote>
                        continue;
                    }
                    ParseStatus::EscapedAtEnd => {
                        // 遇到行尾反斜槓：刪除最後的 \ (已經在 tokenize 邏輯裡沒 push 進去?)
                        // 不，我們的 tokenize 是針對整個 buffer 的。
                        // 如果是 EscapedAtEnd，說明 buffer 最後一個字符是 \
                        // 我們需要把這個 \ 去掉，然後繼續讀下一行
                        if command_buffer.ends_with('\\') {
                            command_buffer.pop(); // 移除 \
                        }
                        continue;
                    }
                    ParseStatus::Error(e) => {
                        eprintln!("Syntax Error: {}", e);
                        command_buffer.clear();
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                command_buffer.clear();
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
        // 每次循環結束保存歷史 (保險起見，也可以放在 break 後)
        let _ = rl.save_history(history_file);
    }

    Ok(())
}
