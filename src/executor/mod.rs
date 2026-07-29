pub mod builtins;
pub mod parser;
pub mod runner;

use crate::config::Config;
use parser::{expand_env_vars, parse_ast, LogicalOperator};
use std::fs;
use std::path::PathBuf;

pub struct Executor {
    history: Vec<String>,
    prev_dir: Option<PathBuf>,
}

impl Executor {
    pub fn new() -> Self {
        let history = Self::load_history();
        Self {
            history,
            prev_dir: None,
        }
    }

    pub fn execute(&mut self, input: &str, config: &mut Config) -> i32 {
        let expanded_env = expand_env_vars(input);
        let trimmed = expanded_env.trim();
        if trimmed.is_empty() {
            return 0;
        }

        let expanded = self.expand_alias(trimmed, config);
        let ast = parse_ast(&expanded);

        if ast.jobs.is_empty() {
            return 0;
        }

        let mut last_code = 0;

        for job in &ast.jobs {
            let code = runner::run_pipeline(
                &job.pipeline,
                &mut self.prev_dir,
                &mut self.history,
                config,
                |hist, prev, cfg, cmd_str| {
                    let start = std::time::Instant::now();
                    let mut temp_exec = Executor {
                        history: hist.clone(),
                        prev_dir: prev.map(|p| p.clone()),
                    };
                    let code = temp_exec.execute(cmd_str, cfg);
                    let elapsed = start.elapsed();
                    println!(
                        "\n Benchmark: command finished in {:.2?} (exit code {})",
                        elapsed, code
                    );
                    code
                },
            );

            last_code = code;

            if let Some(ref op) = job.next_op {
                match op {
                    LogicalOperator::And => {
                        if code != 0 {
                            break;
                        }
                    }
                    LogicalOperator::Or => {
                        if code == 0 {
                            break;
                        }
                    }
                    LogicalOperator::Seq => {
                        // Continue regardless of exit code
                    }
                }
            }
        }

        if last_code == 0 {
            self.add_history(trimmed);
        }

        last_code
    }

    fn expand_alias(&self, input: &str, config: &Config) -> String {
        let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let merged_aliases = config.get_merged_aliases(&current_dir);
        self.expand_alias_recursive(input, &merged_aliases, 0)
    }

    fn expand_alias_recursive(
        &self,
        input: &str,
        aliases: &std::collections::HashMap<String, String>,
        depth: usize,
    ) -> String {
        if depth > 10 {
            return input.to_string();
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        let mut result = String::new();
        let mut current_token = String::new();
        let mut in_single_quote = false;
        let mut in_double_quote = false;

        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];

            if c == '\'' && !in_double_quote {
                in_single_quote = !in_single_quote;
                current_token.push(c);
                i += 1;
                continue;
            }

            if c == '"' && !in_single_quote {
                in_double_quote = !in_double_quote;
                current_token.push(c);
                i += 1;
                continue;
            }

            if !in_single_quote && !in_double_quote && (c == ';' || c == '|' || c == '&') {
                if !current_token.is_empty() {
                    let expanded = self.expand_single_alias_token(&current_token, aliases, depth);
                    result.push_str(&expanded);
                    current_token.clear();
                }
                result.push(c);
                i += 1;
                continue;
            }

            current_token.push(c);
            i += 1;
        }

        if !current_token.is_empty() {
            let expanded = self.expand_single_alias_token(&current_token, aliases, depth);
            result.push_str(&expanded);
        }

        result
    }

    fn expand_single_alias_token(
        &self,
        token_str: &str,
        aliases: &std::collections::HashMap<String, String>,
        depth: usize,
    ) -> String {
        let leading_spaces = token_str.len() - token_str.trim_start().len();
        let trailing_spaces = token_str.len() - token_str.trim_end().len();
        let prefix = &token_str[..leading_spaces];
        let suffix = &token_str[token_str.len() - trailing_spaces..];
        let trimmed = token_str.trim();

        if trimmed.is_empty() {
            return token_str.to_string();
        }

        let mut parts = trimmed.split_whitespace();
        if let Some(first) = parts.next() {
            if let Some(alias_val) = aliases.get(first) {
                let remainder: Vec<&str> = parts.collect();
                let expanded_val = self.expand_alias_recursive(alias_val, aliases, depth + 1);
                let full_cmd = if remainder.is_empty() {
                    expanded_val
                } else {
                    format!("{} {}", expanded_val, remainder.join(" "))
                };
                return format!("{}{}{}", prefix, full_cmd, suffix);
            }
        }

        token_str.to_string()
    }

    pub fn check_update_banner(&self) {
        builtins::sys_cmds::check_update_banner();
    }

    pub fn builtin_update(&self) -> i32 {
        builtins::sys_cmds::builtin_update()
    }

    pub fn builtin_install_wt(&self) -> i32 {
        builtins::sys_cmds::builtin_install_wt()
    }

    pub fn builtin_install_vscode(&self) -> i32 {
        builtins::sys_cmds::builtin_install_vscode()
    }

    #[allow(dead_code)]
    pub fn builtin_install(&self, args: &[String]) -> i32 {
        builtins::sys_cmds::builtin_install(args)
    }

    pub fn builtin_version(&self) -> i32 {
        builtins::sys_cmds::builtin_version()
    }

    #[allow(dead_code)]
    pub fn builtin_tree(&self, args: &[String]) -> i32 {
        builtins::fs_cmds::builtin_tree(args)
    }

    #[allow(dead_code)]
    pub fn builtin_alias(&self, args: &[String], config: &mut Config) -> i32 {
        builtins::shell_cmds::builtin_alias(args, config)
    }

    #[allow(dead_code)]
    pub fn builtin_unalias(&self, args: &[String], config: &mut Config) -> i32 {
        builtins::shell_cmds::builtin_unalias(args, config)
    }

    #[allow(dead_code)]
    pub fn builtin_export(&self, args: &[String]) -> i32 {
        builtins::shell_cmds::builtin_export(args)
    }

    #[allow(dead_code)]
    pub fn builtin_head(&self, args: &[String]) -> i32 {
        builtins::fs_cmds::builtin_head(args)
    }

    #[allow(dead_code)]
    pub fn builtin_tail(&self, args: &[String]) -> i32 {
        builtins::fs_cmds::builtin_tail(args)
    }

    #[allow(dead_code)]
    pub fn builtin_env(&self, args: &[String]) -> i32 {
        builtins::shell_cmds::builtin_env(args)
    }

    #[allow(dead_code)]
    pub fn builtin_unset(&self, args: &[String]) -> i32 {
        builtins::shell_cmds::builtin_unset(args)
    }

    #[allow(dead_code)]
    pub fn builtin_cp(&self, args: &[String]) -> i32 {
        builtins::fs_cmds::builtin_cp(args)
    }

    #[allow(dead_code)]
    pub fn builtin_mv(&self, args: &[String]) -> i32 {
        builtins::fs_cmds::builtin_mv(args)
    }

    fn add_history(&mut self, entry: &str) {
        let trimmed = entry.trim();
        if trimmed.is_empty() {
            return;
        }

        let lower = trimmed.to_lowercase();
        if lower == "history clean"
            || lower == "history clear"
            || lower == "history --clean"
            || lower == "history -c"
            || lower.starts_with("history clean ")
            || lower.starts_with("history clear ")
        {
            return;
        }

        if let Some(last) = self.history.last() {
            if last == trimmed {
                return;
            }
        }

        let first_word = trimmed.split_whitespace().next().unwrap_or("");
        if crate::editor::get_command_status(first_word) == crate::editor::CommandStatus::Invalid {
            return;
        }

        self.history.push(trimmed.to_string());
        self.save_history();
    }

    fn save_history(&self) {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".lshell_history");
            let content = self.history.join("\n");
            let _ = fs::write(path, content);
        }
    }

    fn load_history() -> Vec<String> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".lshell_history");
            if let Ok(content) = fs::read_to_string(path) {
                return content
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| {
                        if s.is_empty() {
                            return false;
                        }
                        let lower = s.to_lowercase();
                        if lower == "history clean"
                            || lower == "history clear"
                            || lower == "history --clean"
                            || lower == "history -c"
                            || lower.starts_with("history clean ")
                            || lower.starts_with("history clear ")
                        {
                            return false;
                        }
                        let first_word = s.split_whitespace().next().unwrap_or("");
                        crate::editor::get_command_status(first_word)
                            != crate::editor::CommandStatus::Invalid
                    })
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_tree_flags() {
        let executor = Executor::new();
        let mut temp_path = env::temp_dir();
        temp_path.push(format!("lshell_tree_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_path);
        fs::create_dir_all(&temp_path).unwrap();

        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(temp_path.join("file1.txt"), "hello").unwrap();
        fs::write(sub_dir.join("file2.txt"), "world").unwrap();

        let orig_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_path).unwrap();

        let res = executor.builtin_tree(&[]);
        assert_eq!(res, 0);
        assert!(!temp_path.join("tree.txt").exists());

        let res = executor.builtin_tree(&["--file".to_string()]);
        assert_eq!(res, 0);
        let file_path = temp_path.join("tree.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("file1.txt"));
        assert!(content.contains("subdir"));
        assert!(!content.contains("file2.txt"));
        fs::remove_file(&file_path).unwrap();

        let res = executor.builtin_tree(&["--full".to_string(), "--file".to_string()]);
        assert_eq!(res, 0);
        assert!(file_path.exists());
        let full_content = fs::read_to_string(&file_path).unwrap();
        assert!(full_content.contains("file1.txt"));
        assert!(full_content.contains("subdir"));
        assert!(full_content.contains("file2.txt"));

        env::set_current_dir(orig_dir).unwrap();
        let _ = fs::remove_dir_all(&temp_path);
    }

    #[test]
    fn test_env_var_expansion() {
        env::set_var("TEST_LSHELL_VAR", "WORLD");
        let expanded = expand_env_vars("echo HELLO_$TEST_LSHELL_VAR");
        assert_eq!(expanded, "echo HELLO_WORLD");
    }

    #[test]
    fn test_alias_and_export() {
        let executor = Executor::new();
        let mut config = Config::default();

        let res = executor.builtin_alias(&["mycls=clear".to_string()], &mut config);
        assert_eq!(res, 0);
        assert_eq!(config.aliases.get("mycls"), Some(&"clear".to_string()));

        let res_dup = executor.builtin_alias(&["mycls=cls".to_string()], &mut config);
        assert_eq!(res_dup, 1);
        assert_eq!(config.aliases.get("mycls"), Some(&"clear".to_string()));

        let res_builtin_block = executor.builtin_alias(&["cd=dir".to_string()], &mut config);
        assert_eq!(res_builtin_block, 1);
        assert_eq!(config.aliases.get("cd"), None);

        let res_unalias_mycls = executor.builtin_unalias(&["mycls".to_string()], &mut config);
        assert_eq!(res_unalias_mycls, 0);
        assert_eq!(config.aliases.get("mycls"), None);

        let res = executor.builtin_export(&["MY_VAR=TEST123".to_string()]);
        assert_eq!(res, 0);
        assert_eq!(env::var("MY_VAR").unwrap(), "TEST123");

        let res_exp_dup = executor.builtin_export(&["MY_VAR=TEST456".to_string()]);
        assert_eq!(res_exp_dup, 1);
        assert_eq!(env::var("MY_VAR").unwrap(), "TEST123");
    }

    #[test]
    fn test_recursive_compound_alias_expansion() {
        let executor = Executor::new();
        let mut config = Config::default();
        config
            .aliases
            .insert("fmt".to_string(), "cargo fmt".to_string());
        config.aliases.insert(
            "clippy".to_string(),
            "cargo clippy --all-targets --all-features".to_string(),
        );
        config
            .aliases
            .insert("checkall".to_string(), "fmt && clippy".to_string());

        let expanded = executor.expand_alias("checkall", &config);
        assert_eq!(
            expanded,
            "cargo fmt && cargo clippy --all-targets --all-features"
        );
    }

    #[test]
    fn test_head_and_tail() {
        let executor = Executor::new();
        let mut temp_path = env::temp_dir();
        temp_path.push(format!("lshell_head_tail_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_path);
        fs::create_dir_all(&temp_path).unwrap();

        let test_file = temp_path.join("lines.txt");
        let lines_str = (1..=20)
            .map(|i| format!("line {}", i))
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&test_file, lines_str).unwrap();

        let res_head = executor.builtin_head(&[
            "-n".to_string(),
            "5".to_string(),
            test_file.to_string_lossy().to_string(),
        ]);
        assert_eq!(res_head, 0);

        let res_tail = executor.builtin_tail(&[
            "-n".to_string(),
            "5".to_string(),
            test_file.to_string_lossy().to_string(),
        ]);
        assert_eq!(res_tail, 0);

        let _ = fs::remove_dir_all(&temp_path);
    }

    #[test]
    fn test_env_and_unset() {
        let executor = Executor::new();
        let res_exp = executor.builtin_export(&["LSHELL_TEST_UNSET=12345".to_string()]);
        assert_eq!(res_exp, 0);
        assert_eq!(env::var("LSHELL_TEST_UNSET").unwrap(), "12345");

        let res_env = executor.builtin_env(&["LSHELL_TEST_UNSET".to_string()]);
        assert_eq!(res_env, 0);

        let res_unset = executor.builtin_unset(&["LSHELL_TEST_UNSET".to_string()]);
        assert_eq!(res_unset, 0);
        assert!(env::var("LSHELL_TEST_UNSET").is_err());
    }

    #[test]
    fn test_cp_and_mv() {
        let executor = Executor::new();
        let mut temp_path = env::temp_dir();
        temp_path.push(format!("lshell_cp_mv_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_path);
        fs::create_dir_all(&temp_path).unwrap();

        let src_file = temp_path.join("src.txt");
        let cp_file = temp_path.join("cp.txt");
        let mv_file = temp_path.join("mv.txt");

        fs::write(&src_file, "copy test content").unwrap();

        let res_cp = executor.builtin_cp(&[
            src_file.to_string_lossy().to_string(),
            cp_file.to_string_lossy().to_string(),
        ]);
        assert_eq!(res_cp, 0);
        assert!(cp_file.exists());
        assert_eq!(fs::read_to_string(&cp_file).unwrap(), "copy test content");

        let res_mv = executor.builtin_mv(&[
            cp_file.to_string_lossy().to_string(),
            mv_file.to_string_lossy().to_string(),
        ]);
        assert_eq!(res_mv, 0);
        assert!(!cp_file.exists());
        assert!(mv_file.exists());
        assert_eq!(fs::read_to_string(&mv_file).unwrap(), "copy test content");

        let _ = fs::remove_dir_all(&temp_path);
    }

    #[test]
    fn test_history_clean() {
        let mut history = vec!["cargo run".to_string(), "carggo run".to_string()];
        let res = builtins::shell_cmds::builtin_history(&mut history, &["clean".to_string()]);
        assert_eq!(res, 0);
        assert!(history.is_empty());
    }

    #[test]
    fn test_history_add_validation() {
        let mut executor = Executor::new();
        executor.history.clear();

        executor.add_history("history clean");
        assert!(executor.history.is_empty());

        executor.add_history("carggo_invalid_xyz");
        assert!(executor.history.is_empty());

        executor.add_history("cargo run");
        assert_eq!(executor.history.len(), 1);

        executor.add_history("cargo run");
        assert_eq!(executor.history.len(), 1);
    }
}
