use chrono::Duration;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
use sysinfo::{CpuRefreshKind, ProcessRefreshKind, System, RefreshKind, MemoryRefreshKind, Disks};

struct ShellState {
    prev_dir: Option<PathBuf>,
    sys: System,
    history_path: PathBuf,
}

impl ShellState {
    fn new() -> Self {
        let history_path = dirs::home_dir()
            .map(|mut p| { p.push(".shyell_history"); p })
            .unwrap_or_else(|| PathBuf::from(".shyell_history"));

        Self {
            prev_dir: None,
            sys: System::new_with_specifics(RefreshKind::nothing()),
            history_path,
        }
    }

    fn execute_builtins(&mut self, cmd: &CommandExecution) -> bool {
        if cmd.args.is_empty() {
            return false;
        }
        
        let command = cmd.args[0].as_str();
        
        match command {
            "cd" => {
                let target = if cmd.args.len() < 2 {
                    dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"))
                } else if cmd.args[1] == "-" {
                    if let Some(prev) = &self.prev_dir {
                        prev.clone()
                    } else {
                        eprintln!("cd: oldpwd not set");
                        return true;
                    }
                } else {
                    PathBuf::from(&cmd.args[1])
                };
                
                let current = env::current_dir().unwrap_or_default();
                match env::set_current_dir(&target) {
                    Ok(_) => {
                        self.prev_dir = Some(current);
                    }
                    Err(e) => eprintln!("cd: {}: {}", target.display(), e),
                }
                true
            }
            "pwd" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("shyell: {}", e); return true; }
                };
                match env::current_dir() {
                    Ok(dir) => writeln!(writer, "{}", dir.display()).unwrap_or(()),
                    Err(e) => eprintln!("pwd: {}", e),
                }
                true
            }
            "sys" => {
                self.sys.refresh_specifics(
                    RefreshKind::nothing()
                        .with_cpu(CpuRefreshKind::everything())
                        .with_memory(MemoryRefreshKind::everything())
                );
                
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("shyell: {}", e); return true; }
                };
                
                writeln!(writer, "\x1b[1;36m--- System Status ---\x1b[0m").unwrap_or(());
                writeln!(writer, "{:<15} {}", "OS:", System::name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Kernel:", System::kernel_version().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Hostname:", System::host_name().unwrap_or_else(|| "Unknown".into())).unwrap_or(());
                writeln!(writer, "{:<15} {}", "Uptime:", format_duration(System::uptime())).unwrap_or(());
                
                let total_mem = self.sys.total_memory() / 1024 / 1024;
                let used_mem = self.sys.used_memory() / 1024 / 1024;
                let mem_pct = (used_mem as f64 / total_mem as f64 * 100.0) as usize;
                
                let bar_len = 20;
                let filled = (mem_pct * bar_len) / 100;
                let bar = format!("[{}{}]", "#".repeat(filled), ".".repeat(bar_len - filled));
                
                writeln!(writer, "{:<15} {} {}% ({}MB / {}MB)", "Memory:", bar, mem_pct, used_mem, total_mem).unwrap_or(());
                writeln!(writer, "{:<15} {:.2}%", "CPU Load:", self.sys.global_cpu_usage()).unwrap_or(());

                // Disk Info
                let disks = Disks::new_with_refreshed_list();
                if let Some(root) = disks.iter().find(|d| d.mount_point() == std::path::Path::new("/")) {
                    let total = root.total_space() / 1024 / 1024 / 1024;
                    let avail = root.available_space() / 1024 / 1024 / 1024;
                    let used = total - avail;
                    let disk_pct = (used as f64 / total as f64 * 100.0) as usize;
                    let filled_disk = (disk_pct * bar_len) / 100;
                    let bar_disk = format!("[{}{}]", "#".repeat(filled_disk), ".".repeat(bar_len - filled_disk));
                    writeln!(writer, "{:<15} {} {}% ({}GB / {}GB)", "Disk (/):", bar_disk, disk_pct, used, total).unwrap_or(());
                }
                
                true
            }
            "top" => {
                self.sys.refresh_specifics(
                    RefreshKind::nothing().with_processes(ProcessRefreshKind::everything())
                );
                
                let mut processes: Vec<_> = self.sys.processes().values().collect();
                processes.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap());
                
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("shyell: {}", e); return true; }
                };
                
                writeln!(writer, "\x1b[1;33m{:<8} {:<15} {:<10} {:<10}\x1b[0m", "PID", "Name", "CPU %", "Mem MB").unwrap_or(());
                for p in processes.iter().take(10) {
                    writeln!(writer, "{:<8} {:<15} {:<10.2} {:<10}", 
                        p.pid(), 
                        p.name().to_string_lossy(), 
                        p.cpu_usage(), 
                        p.memory() / 1024 / 1024
                    ).unwrap_or(());
                }
                true
            }
            "help" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("shyell: {}", e); return true; }
                };
                writeln!(writer, "\x1b[1;32mShyell - Advanced Performance Shell\x1b[0m").unwrap_or(());
                writeln!(writer, "\x1b[1mSystem Performance:\x1b[0m").unwrap_or(());
                writeln!(writer, "  sys         Show system overview (CPU, Mem, Disk, Uptime).").unwrap_or(());
                writeln!(writer, "  top         Show top 10 processes by CPU usage.").unwrap_or(());
                writeln!(writer, "  bench <cmd> Prefix any command to measure its time/resources.").unwrap_or(());
                writeln!(writer, "\x1b[1mStandard Commands:\x1b[0m").unwrap_or(());
                writeln!(writer, "  cd [dir]    Change directory ('cd -' for back).").unwrap_or(());
                writeln!(writer, "  pwd         Print current directory.").unwrap_or(());
                writeln!(writer, "  echo        Print arguments.").unwrap_or(());
                writeln!(writer, "  exit        Exit the shell.").unwrap_or(());
                true
            }
            "echo" => {
                let mut writer = match get_output_writer(&cmd.output_file, cmd.append) {
                    Ok(w) => w,
                    Err(e) => { eprintln!("shyell: {}", e); return true; }
                };
                let output = cmd.args[1..].join(" ");
                writeln!(writer, "{}", output).unwrap_or(());
                true
            }
            "exit" => {
                std::process::exit(0);
            }
            _ => false,
        }
    }
}

struct CommandExecution {
    args: Vec<String>,
    input_file: Option<String>,
    output_file: Option<String>,
    append: bool,
    bench: bool,
}

fn expand_word(word: &str) -> String {
    let mut expanded = String::new();
    if word.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            expanded.push_str(&home.to_string_lossy());
            expanded.push_str(&word[1..]);
        } else {
            expanded.push_str(word);
        }
    } else {
        expanded.push_str(word);
    }
    
    let mut result = String::new();
    let mut chars = expanded.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '$' {
            let mut var = String::new();
            if let Some(&'{') = chars.peek() {
                chars.next(); // consume '{'
                while let Some(&nc) = chars.peek() {
                    if nc == '}' {
                        chars.next(); // consume '}'
                        break;
                    }
                    var.push(nc);
                    chars.next();
                }
            } else {
                while let Some(&nc) = chars.peek() {
                    if nc.is_alphanumeric() || nc == '_' {
                        var.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            
            if !var.is_empty() {
                if let Ok(val) = env::var(&var) {
                    result.push_str(&val);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(c);
        }
    }
    result
}

fn parse_commands(words: Vec<String>) -> Vec<CommandExecution> {
    let mut cmds = Vec::new();
    let mut is_bench = false;
    let mut start_idx = 0;

    if !words.is_empty() && words[0] == "bench" {
        is_bench = true;
        start_idx = 1;
    }

    let mut current_cmd = CommandExecution {
        args: Vec::new(),
        input_file: None,
        output_file: None,
        append: false,
        bench: is_bench,
    };
    
    let mut iter = words.into_iter().skip(start_idx).peekable();
    while let Some(word) = iter.next() {
        match word.as_str() {
            "|" => {
                cmds.push(current_cmd);
                current_cmd = CommandExecution {
                    args: Vec::new(),
                    input_file: None,
                    output_file: None,
                    append: false,
                    bench: is_bench,
                };
            }
            ">" => {
                if let Some(file) = iter.next() {
                    current_cmd.output_file = Some(expand_word(&file));
                }
            }
            ">>" => {
                if let Some(file) = iter.next() {
                    current_cmd.output_file = Some(expand_word(&file));
                    current_cmd.append = true;
                }
            }
            "<" => {
                if let Some(file) = iter.next() {
                    current_cmd.input_file = Some(expand_word(&file));
                }
            }
            _ => {
                current_cmd.args.push(expand_word(&word));
            }
        }
    }
    if !current_cmd.args.is_empty() || current_cmd.input_file.is_some() || current_cmd.output_file.is_some() {
        cmds.push(current_cmd);
    }
    cmds
}

fn get_output_writer(output_file: &Option<String>, append: bool) -> Result<Box<dyn Write>, String> {
    if let Some(file) = output_file {
        let f = if append {
            std::fs::OpenOptions::new().create(true).append(true).open(file)
        } else {
            File::create(file)
        };
        match f {
            Ok(f) => Ok(Box::new(f)),
            Err(e) => Err(format!("{}: {}", file, e)),
        }
    } else {
        Ok(Box::new(std::io::stdout()))
    }
}

fn format_duration(seconds: u64) -> String {
    let d = Duration::seconds(seconds as i64);
    let hours = d.num_hours();
    let mins = d.num_minutes() % 60;
    let secs = d.num_seconds() % 60;
    if hours > 0 {
        format!("{}h {}m {}s", hours, mins, secs)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    }
}

fn execute_commands(cmds: Vec<CommandExecution>, state: &mut ShellState) {
    if cmds.is_empty() {
        return;
    }

    let is_bench = cmds[0].bench;
    let start_time = if is_bench { Some(Instant::now()) } else { None };

    if cmds.len() == 1 && state.execute_builtins(&cmds[0]) {
        if let Some(start) = start_time {
            println!("\x1b[1;35mBench: Built-in command took {:?}\x1b[0m", start.elapsed());
        }
        return;
    }

    let mut children = Vec::new();
    let mut previous_stdout: Option<Stdio> = None;
    let cmd_count = cmds.len();

    for (i, cmd_exec) in cmds.iter().enumerate() {
        if cmd_exec.args.is_empty() {
            eprintln!("shyell: parse error: empty command in pipeline");
            return;
        }

        let stdin = if let Some(ref in_file) = cmd_exec.input_file {
            match File::open(in_file) {
                Ok(f) => Stdio::from(f),
                Err(e) => {
                    eprintln!("shyell: {}: {}", in_file, e);
                    return;
                }
            }
        } else if let Some(prev) = previous_stdout.take() {
            prev
        } else {
            Stdio::inherit()
        };

        let stdout = if let Some(ref out_file) = cmd_exec.output_file {
            let f = if cmd_exec.append {
                std::fs::OpenOptions::new().create(true).append(true).open(out_file)
            } else {
                File::create(out_file)
            };
            match f {
                Ok(f) => Stdio::from(f),
                Err(e) => {
                    eprintln!("shyell: {}: {}", out_file, e);
                    return;
                }
            }
        } else if i < cmd_count - 1 {
            Stdio::piped()
        } else {
            Stdio::inherit()
        };

        let command = &cmd_exec.args[0];
        let args = &cmd_exec.args[1..];

        let mut cmd = Command::new(command);
        cmd.args(args);
        cmd.stdin(stdin);
        cmd.stdout(stdout);

        match cmd.spawn() {
            Ok(mut child) => {
                if i < cmd_count - 1 {
                    previous_stdout = Some(Stdio::from(child.stdout.take().unwrap()));
                }
                children.push((command.clone(), child));
            }
            Err(e) => {
                eprintln!("shyell: {}: {}", command, e);
                // Even if one fails, we should wait for the already spawned ones
                break;
            }
        }
    }

    let mut last_status = None;
    for (name, mut child) in children {
        match child.wait() {
            Ok(s) => last_status = Some(s),
            Err(e) => eprintln!("shyell: error waiting for {}: {}", name, e),
        }
    }

    if let Some(start) = start_time {
        let elapsed = start.elapsed();
        println!("\x1b[1;35m--- Benchmark Results ---\x1b[0m");
        println!("{:<15} {:?}", "Execution Time:", elapsed);
        if let Some(s) = last_status {
            println!("{:<15} {}", "Exit Status:", s);
        }
    }
}

fn get_prompt() -> String {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("?"));
    let mut cwd_str = cwd.to_string_lossy().to_string();
    if let Some(home) = dirs::home_dir() {
        let home_str = home.to_string_lossy().to_string();
        if cwd_str.starts_with(&home_str) {
            cwd_str = cwd_str.replacen(&home_str, "~", 1);
        }
    }
    let user = env::var("USER").unwrap_or_else(|_| "user".to_string());
    // Green user, blue path, bright white prompt
    format!("\x1b[1;32m{}\x1b[0m:\x1b[1;34m{}\x1b[0m\x1b[1;37m$ \x1b[0m", user, cwd_str)
}

fn main() {
    let mut state = ShellState::new();
    let mut rl = DefaultEditor::new().unwrap();

    let _ = rl.load_history(&state.history_path);

    loop {
        let readline = rl.readline(&get_prompt());
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(line);
                
                match shell_words::split(line) {
                    Ok(words) => {
                        let cmds = parse_commands(words);
                        execute_commands(cmds, &mut state);
                    }
                    Err(e) => eprintln!("Parse error: {}", e),
                }
            },
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => {
                println!("exit");
                break;
            },
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    let _ = rl.save_history(&state.history_path);
}
