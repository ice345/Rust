# Rust Vim-like Editor

<!--toc:start-->
- [Rust Vim-like Editor](#rust-vim-like-editor)
  - [Project Goals](#project-goals)
  - [Project Structure](#project-structure)
  - [Core Concepts & Design](#core-concepts-design)
    - [State Management](#state-management)
  - [Using the `crossterm` Crate](#using-the-crossterm-crate)
    - [Key Concepts in `crossterm`](#key-concepts-in-crossterm)
  - [How to Build and Run](#how-to-build-and-run)
    - [Prerequisites](#prerequisites)
    - [Build](#build)
    - [Run](#run)
    - [Test](#test)
<!--toc:end-->

This is a minimal, Vim-like text editor built in Rust. It serves as a learning project to explore terminal UI manipulation, application architecture in Rust, and modern tooling.

## Project Goals

- **Modularity**: Structure the code logically into a library and a binary.
- **Testability**: Implement both unit and integration tests to ensure code quality.
- **Clarity**: Maintain clean, well-documented code that follows Rust best practices.

---

## Project Structure

The project is organized using the recommended library + binary pattern in Rust, which enhances modularity and testability.

```shell
.
├── Cargo.toml
├── src
│   ├── main.rs         # Binary entry point: launches the editor
│   ├── lib.rs          # Library root: defines the public API and modules
│   ├── editor.rs       # Core editor logic, state machine, and event loop
│   ├── output.rs       # Handles all screen rendering and drawing
│   ├── cursor.rs       # Manages cursor position and screen scrolling
│   ├── editor_rows.rs  # Manages file content, text rows, and file I/O
│   └── ...             # Other supporting modules
└── tests
    ├── common/         # Test utilities and helpers
    └── file_operations.rs # Integration tests for file I/O
```

- **`src/main.rs`**: The binary crate. Its only job is to call the library's `run()` function. This keeps the entry point clean and simple.
- **`src/lib.rs`**: The library crate. It contains all the core application logic, making it reusable and, most importantly, **testable** by integration tests.
- **`tests/`**: The integration tests directory. Cargo treats each file here as a separate crate that tests the public API of our library.

---

## Core Concepts & Design

The editor operates on a simple loop:

1. **Refresh Screen**: Draw the current state of the UI (text, status bar, cursor) to the terminal.
2. **Process Keypress**: Wait for and handle user input.
3. **Update State**: Modify the editor's state (e.g., switch mode, move cursor, edit text) based on the input.
4. **Repeat**.

### State Management

The editor is a state machine, primarily managed by the `Mode` enum (`Normal`, `Insert`, `Command`, `Search`). The `Editor` struct holds the current state, and the `process_keypress` function is the main driver for state transitions.

---

## Using the `crossterm` Crate

`crossterm` is a powerful, cross-platform terminal manipulation library. It's the backbone of this editor, allowing us to control the terminal directly.

### Key Concepts in `crossterm`

1. **Raw Mode (裸模式)**

    - **What it is**: Normally, the terminal processes input line by line. It also interprets special key combinations (like `Ctrl+C`). Raw mode disables all of this, allowing our application to receive every single keypress event (e.g., `h`, `j`, `k`, `l`, arrow keys) instantly and directly.
    - **How we use it**: We enable raw mode at the very start of the application with `terminal::enable_raw_mode()?`. To ensure the terminal state is always restored, we use a simple RAII (Resource Acquisition Is Initialization) struct (`cleanup::CleanUp`) that automatically disables raw mode when it goes out of scope (e.g., on exit or panic).

2. **Command Queue (`queue!`)**

    - **What it is**: Performing many separate terminal operations (e.g., move cursor, change color, print text) can be inefficient, as each one might make a system call. `crossterm` provides a `queue!` macro to batch multiple commands together.
    - **How we use it**: In the `refresh_screen` function, we queue all drawing commands (moving the cursor, clearing lines, printing text) into a buffer (`EditorContents`). The commands are only sent to the terminal and executed at the very end when we call `flush()` on the buffer. This minimizes I/O operations and reduces screen flicker.

3. **Core `crossterm` APIs Used**

    - `terminal::enable_raw_mode()` / `disable_raw_mode()`: To enter and exit raw mode.
    - `terminal::size()`: To get the current dimensions of the terminal window.
    - `terminal::Clear()`: To clear parts of the screen.
    - `cursor::MoveTo()`: To position the cursor at a specific `(x, y)` coordinate.
    - `cursor::Hide` / `cursor::Show`: To control cursor visibility during redraws, preventing flicker.
    - `style::Print()`: To print text to the screen.
    - `event::read()`: To read a single input event (like a keypress).

---

## How to Build and Run

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)

### Build

```bash
# Build the project in debug mode
cargo build
```

### Run

```bash
# Run the editor and open a file
cargo run -- path/to/your/file.txt

# Run without opening a file
cargo run
```

### Test

```bash
# Run all unit and integration tests
cargo test
```

