# Shyell-in-Rust 🦀🐚

**Shyell** is a modern, performance-focused Linux shell implemented in Rust. It combines the core functionality of a traditional shell with built-in system monitoring and benchmarking tools.

## ✨ Key Features

### 🚀 Performance Dashboard
Unlike traditional shells, Shyell has native support for system health monitoring:
- **`sys`**: Displays a colorized snapshot of your system (OS, Kernel, Uptime, CPU Load, and a visual Memory bar).
- **`top`**: A built-in process monitor showing the top 10 resource-hungry processes.
- **`bench`**: A prefix command to measure the exact execution time and resources of any command (e.g., `bench cargo build`).

### 🛠️ Core Shell Features
- **Piping & Redirection**: Support for complex pipelines (`ls | grep rs | wc -l`) and file redirection (`>`, `>>`, `<`).
- **Variable Expansion**: Support for environment variables (e.g., `echo $USER`) and home directory expansion (`~`).
- **Built-in Commands**: Optimized versions of `cd` (with `cd -` support), `pwd`, `echo`, `help`, and `exit`.
- **Line Editing & History**: Full support for arrow keys, Ctrl+C/D, and persistent command history (saved to `~/.shyell_history`).

### 🎨 Modern UI
- **Colorized Prompt**: A professional, easy-to-read prompt showing `user:path$`.
- **ANSI Color Support**: Colorized output for system status and help menus.

## 📦 Installation

Ensure you have [Rust](https://www.rust-lang.org/tools/install) installed.

1. Clone the repository:
   ```bash
   git clone https://github.com/sumant/Shyell-in-Rust.git
   cd Shyell-in-Rust
   ```

2. Run the shell:
   ```bash
   cargo run
   ```

## 📖 Usage Examples

### System Monitoring
```bash
user:~$ sys
user:~$ top
```

### Benchmarking
```bash
user:~$ bench sleep 2
--- Benchmark Results ---
Execution Time: 2.001s
Exit Status: Success
```

### Pipelines and Files
```bash
user:~$ ls -l | grep .rs > rust_files.txt
user:~$ cat < rust_files.txt
```

## 📚 Built-in Commands
| Command | Description |
|---------|-------------|
| `cd [dir]` | Change directory (`cd` for HOME, `cd -` for back) |
| `pwd` | Print current working directory |
| `echo` | Print arguments to stdout |
| `sys` | Show system status snapshot |
| `top` | Show top CPU-using processes |
| `bench <cmd>` | Benchmark a command's performance |
| `help` | Show the help menu |
| `exit` | Exit Shyell |

## 🛠️ Built With
- [rustyline](https://crates.io/crates/rustyline) - For line editing and history.
- [sysinfo](https://crates.io/crates/sysinfo) - For system and process metrics.
- [shell-words](https://crates.io/crates/shell-words) - For robust command parsing.
- [dirs](https://crates.io/crates/dirs) - For cross-platform path management.
- [chrono](https://crates.io/crates/chrono) - For time formatting.
