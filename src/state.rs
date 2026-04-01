use std::env;
use std::path::PathBuf;
use sysinfo::{System, RefreshKind};
use serde::{Serialize, Deserialize};
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BenchmarkResult {
    pub command: String,
    pub duration_secs: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub exit_status: Option<i32>,
}

pub struct ShellState {
    pub prev_dir: Option<PathBuf>,
    pub sys: System,
    pub history_path: PathBuf,
    pub bench_history_path: PathBuf,
    pub bench_results: Vec<BenchmarkResult>,
    pub last_exit_status: Option<i32>,
}

impl ShellState {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let history_path = home.join(".vantage_history");
        let bench_history_path = home.join(".vantage_benchmarks.json");

        let bench_results = if let Ok(content) = fs::read_to_string(&bench_history_path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Self {
            prev_dir: None,
            sys: System::new_with_specifics(RefreshKind::nothing()),
            history_path,
            bench_history_path,
            bench_results,
            last_exit_status: Some(0), // Default to success
        }
    }

    pub fn save_benchmarks(&self) {
        if let Ok(content) = serde_json::to_string_pretty(&self.bench_results) {
            let _ = fs::write(&self.bench_history_path, content);
        }
    }

    pub fn add_benchmark(&mut self, command: String, duration_secs: f64, exit_status: Option<i32>) {
        let result = BenchmarkResult {
            command,
            duration_secs,
            timestamp: chrono::Utc::now(),
            exit_status,
        };
        self.bench_results.push(result);
        self.save_benchmarks();
    }
}

pub fn expand_word(word: &str) -> String {
    let mut expanded = String::new();
    if let Some(rest) = word.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            expanded.push_str(&home.to_string_lossy());
            expanded.push_str(rest);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_expand_word_no_expansion() {
        assert_eq!(expand_word("hello"), "hello");
    }

    #[test]
    fn test_expand_word_tilde() {
        let home = env::var("HOME").unwrap();
        assert_eq!(expand_word("~/test"), format!("{}/test", home));
    }

    #[test]
    fn test_expand_word_env_var() {
        unsafe { env::set_var("TEST_VAR", "value"); }
        assert_eq!(expand_word("$TEST_VAR"), "value");
        assert_eq!(expand_word("${TEST_VAR}"), "value");
        assert_eq!(expand_word("prefix_$TEST_VAR"), "prefix_value");
    }

    #[test]
    fn test_expand_word_mixed() {
        unsafe { env::set_var("FOO", "bar"); }
        let home = env::var("HOME").unwrap();
        assert_eq!(expand_word("~/$FOO"), format!("{}/bar", home));
    }

    #[test]
    fn test_expand_word_empty_var() {
        assert_eq!(expand_word("$NON_EXISTENT_VAR"), "");
    }
}
