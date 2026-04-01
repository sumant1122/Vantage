use crate::state::expand_word;

#[derive(Debug, Clone)]
pub struct CommandExecution {
    pub args: Vec<String>,
    pub input_file: Option<String>,
    pub output_file: Option<String>,
    pub append: bool,
    pub bench: bool,
}

pub fn parse_commands(words: Vec<String>) -> Vec<CommandExecution> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_command() {
        let words = vec!["ls".to_string(), "-la".to_string()];
        let cmds = parse_commands(words);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].args, vec!["ls", "-la"]);
        assert!(!cmds[0].bench);
    }

    #[test]
    fn test_parse_bench() {
        let words = vec!["bench".to_string(), "ls".to_string()];
        let cmds = parse_commands(words);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].args, vec!["ls"]);
        assert!(cmds[0].bench);
    }

    #[test]
    fn test_parse_pipeline() {
        let words = vec!["ls".to_string(), "|".to_string(), "grep".to_string(), "rs".to_string()];
        let cmds = parse_commands(words);
        assert_eq!(cmds.len(), 2);
        assert_eq!(cmds[0].args, vec!["ls"]);
        assert_eq!(cmds[1].args, vec!["grep", "rs"]);
    }

    #[test]
    fn test_parse_redirection() {
        let words = vec!["cat".to_string(), "<".to_string(), "input.txt".to_string(), ">".to_string(), "output.txt".to_string()];
        let cmds = parse_commands(words);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].args, vec!["cat"]);
        assert_eq!(cmds[0].input_file, Some("input.txt".to_string()));
        assert_eq!(cmds[0].output_file, Some("output.txt".to_string()));
        assert!(!cmds[0].append);
    }

    #[test]
    fn test_parse_append_redirection() {
        let words = vec!["echo".to_string(), "hi".to_string(), ">>".to_string(), "log.txt".to_string()];
        let cmds = parse_commands(words);
        assert_eq!(cmds.len(), 1);
        assert_eq!(cmds[0].args, vec!["echo", "hi"]);
        assert_eq!(cmds[0].output_file, Some("log.txt".to_string()));
        assert!(cmds[0].append);
    }
}
