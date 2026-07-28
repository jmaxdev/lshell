pub mod interactive;
pub use interactive::{choice_multi, choice_single, input_text};

use crate::config::Config;
use crossterm::{
    cursor::MoveToColumn,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{stdout, Write};

pub struct LineEditor;

impl LineEditor {
    pub fn read_line(
        prompt: &str,
        history: &[String],
        config: &Config,
    ) -> Result<String, Box<dyn std::error::Error>> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        print!("{}", prompt);
        stdout.flush()?;

        let mut buffer = String::new();
        let mut cursor_pos: usize = 0;
        let mut history_index: usize = history.len();
        let mut search_mode = false;
        let mut search_query = String::new();

        loop {
            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }

                let KeyEvent {
                    code, modifiers, ..
                } = key_event;

                if code == KeyCode::Char('r') && modifiers.contains(KeyModifiers::CONTROL) {
                    search_mode = !search_mode;
                    search_query.clear();
                }

                // Compute autosuggestion BEFORE handling KeyCode::Right or KeyCode::Tab, or compute after keypress
                let current_autosuggestion = if config.enable_autosuggestions
                    && buffer.len() >= config.min_autosuggestion_len
                    && !search_mode
                    && cursor_pos == buffer.len()
                {
                    history
                        .iter()
                        .rev()
                        .find(|cmd| {
                            let trimmed_cmd = cmd.trim();
                            if !trimmed_cmd
                                .to_lowercase()
                                .starts_with(&buffer.to_lowercase())
                                || trimmed_cmd.eq_ignore_ascii_case(&buffer)
                                || trimmed_cmd.eq_ignore_ascii_case(buffer.trim())
                            {
                                return false;
                            }
                            let first_word = trimmed_cmd.split_whitespace().next().unwrap_or("");
                            get_command_status(first_word) == CommandStatus::Valid
                        })
                        .map(|cmd| cmd[buffer.len()..].to_string())
                } else {
                    None
                };

                if search_mode {
                    match code {
                        KeyCode::Esc | KeyCode::Enter => {
                            if !search_query.is_empty() {
                                if let Some(found) =
                                    history.iter().rev().find(|c| c.contains(&search_query))
                                {
                                    buffer = found.clone();
                                    cursor_pos = buffer.len();
                                }
                            }
                            search_mode = false;
                        }
                        KeyCode::Backspace => {
                            search_query.pop();
                        }
                        KeyCode::Char(c) => {
                            search_query.push(c);
                        }
                        _ => {}
                    }
                } else {
                    match code {
                        KeyCode::Enter => {
                            println!();
                            disable_raw_mode()?;
                            return Ok(buffer);
                        }

                        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                            buffer.clear();
                            cursor_pos = 0;
                            println!("^C");
                            print!("{}", prompt);
                            stdout.flush()?;
                        }

                        KeyCode::Char('d')
                            if modifiers.contains(KeyModifiers::CONTROL) && buffer.is_empty() =>
                        {
                            println!("exit");
                            disable_raw_mode()?;
                            return Ok("exit".to_string());
                        }

                        KeyCode::Right => {
                            if cursor_pos < buffer.len() {
                                if let Some(ch) = buffer[cursor_pos..].chars().next() {
                                    cursor_pos += ch.len_utf8();
                                }
                            } else if let Some(ref suggest) = current_autosuggestion {
                                buffer.push_str(suggest);
                                cursor_pos = buffer.len();
                            }
                        }

                        KeyCode::Left if cursor_pos > 0 => {
                            if let Some(ch) = buffer[..cursor_pos].chars().next_back() {
                                cursor_pos -= ch.len_utf8();
                            }
                        }

                        KeyCode::Backspace if cursor_pos > 0 => {
                            if let Some(ch) = buffer[..cursor_pos].chars().next_back() {
                                let ch_len = ch.len_utf8();
                                cursor_pos -= ch_len;
                                buffer.remove(cursor_pos);
                            }
                        }

                        KeyCode::Delete if cursor_pos < buffer.len() => {
                            buffer.remove(cursor_pos);
                        }

                        KeyCode::Home => {
                            cursor_pos = 0;
                        }
                        KeyCode::Char('a') if modifiers.contains(KeyModifiers::CONTROL) => {
                            cursor_pos = 0;
                        }

                        KeyCode::End => {
                            cursor_pos = buffer.len();
                        }
                        KeyCode::Char('e') if modifiers.contains(KeyModifiers::CONTROL) => {
                            cursor_pos = buffer.len();
                        }

                        KeyCode::Up if !history.is_empty() && history_index > 0 => {
                            history_index -= 1;
                            buffer = history[history_index].clone();
                            cursor_pos = buffer.len();
                        }

                        KeyCode::Down => {
                            if !history.is_empty() && history_index + 1 < history.len() {
                                history_index += 1;
                                buffer = history[history_index].clone();
                                cursor_pos = buffer.len();
                            } else {
                                history_index = history.len();
                                buffer.clear();
                                cursor_pos = 0;
                            }
                        }

                        KeyCode::Tab => {
                            if let Some(completed) = autocomplete_command_or_file(&buffer) {
                                buffer = completed;
                                cursor_pos = buffer.len();
                            } else if let Some(ref suggest) = current_autosuggestion {
                                buffer.push_str(suggest);
                                cursor_pos = buffer.len();
                            }
                        }

                        KeyCode::Char(c) => {
                            buffer.insert(cursor_pos, c);
                            cursor_pos += c.len_utf8();
                        }

                        _ => {}
                    }
                }

                // Recalculate autosuggestion AFTER buffer modification for rendering
                let autosuggestion = if config.enable_autosuggestions
                    && buffer.len() >= config.min_autosuggestion_len
                    && !search_mode
                    && cursor_pos == buffer.len()
                {
                    history
                        .iter()
                        .rev()
                        .find(|cmd| {
                            let trimmed_cmd = cmd.trim();
                            if !trimmed_cmd
                                .to_lowercase()
                                .starts_with(&buffer.to_lowercase())
                                || trimmed_cmd.eq_ignore_ascii_case(&buffer)
                                || trimmed_cmd.eq_ignore_ascii_case(buffer.trim())
                            {
                                return false;
                            }
                            let first_word = trimmed_cmd.split_whitespace().next().unwrap_or("");
                            get_command_status(first_word) == CommandStatus::Valid
                        })
                        .map(|cmd| cmd[buffer.len()..].to_string())
                } else {
                    None
                };

                queue!(
                    stdout,
                    MoveToColumn(0),
                    Clear(ClearType::UntilNewLine),
                    SetForegroundColor(Color::AnsiValue(78)),
                    Print(&config.prompt_symbol),
                    ResetColor
                )?;

                if search_mode {
                    let match_cmd = history
                        .iter()
                        .rev()
                        .find(|c| c.contains(&search_query))
                        .cloned()
                        .unwrap_or_else(|| "no matches found".to_string());

                    queue!(
                        stdout,
                        SetForegroundColor(Color::AnsiValue(214)),
                        Print(format!("(search '{}'): {}", search_query, match_cmd)),
                        ResetColor
                    )?;
                } else {
                    let (cmd_part, rest_part) = match buffer.split_once(' ') {
                        Some((cmd, rest)) => (cmd, format!(" {}", rest)),
                        None => (buffer.as_str(), String::new()),
                    };

                    if config.enable_syntax_highlighting {
                        let status = get_command_status(cmd_part);
                        let cmd_color = match status {
                            CommandStatus::Valid => Color::AnsiValue(78), // Green for exact valid command
                            CommandStatus::Prefix => Color::AnsiValue(255), // White for typing valid command prefix
                            CommandStatus::Invalid => Color::AnsiValue(203), // Red for invalid command
                        };

                        queue!(
                            stdout,
                            SetForegroundColor(cmd_color),
                            Print(cmd_part),
                            ResetColor
                        )?;
                    } else {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::AnsiValue(255)),
                            Print(cmd_part),
                            ResetColor
                        )?;
                    }

                    queue!(
                        stdout,
                        SetForegroundColor(Color::AnsiValue(252)),
                        Print(rest_part),
                        ResetColor
                    )?;

                    if let Some(ref suggest) = autosuggestion {
                        queue!(
                            stdout,
                            SetAttribute(Attribute::Italic),
                            SetForegroundColor(Color::AnsiValue(242)),
                            Print(suggest),
                            SetAttribute(Attribute::NoItalic),
                            ResetColor
                        )?;
                    }
                }

                let prompt_prefix_len = config.prompt_symbol.chars().count();
                let char_count = buffer[..cursor_pos].chars().count();
                let target_col = prompt_prefix_len + char_count;
                queue!(stdout, MoveToColumn(target_col as u16))?;
                stdout.flush()?;
            }
        }
    }
}

use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant};

struct PathCache {
    binaries: HashSet<String>,
    last_updated: Instant,
}

static PATH_CACHE: Mutex<Option<PathCache>> = Mutex::new(None);

fn get_path_binaries() -> HashSet<String> {
    if let Ok(mut cache_guard) = PATH_CACHE.lock() {
        if let Some(ref cache) = *cache_guard {
            if cache.last_updated.elapsed() < Duration::from_secs(10) {
                return cache.binaries.clone();
            }
        }

        let mut set = HashSet::new();
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if let Ok(entries) = std::fs::read_dir(dir) {
                    for entry in entries.flatten() {
                        let name = entry.file_name().to_string_lossy().to_lowercase();
                        let clean_name = if let Some(stripped) = name.strip_suffix(".exe") {
                            stripped.to_string()
                        } else if let Some(stripped) = name.strip_suffix(".bat") {
                            stripped.to_string()
                        } else if let Some(stripped) = name.strip_suffix(".cmd") {
                            stripped.to_string()
                        } else {
                            name.clone()
                        };
                        set.insert(clean_name);
                        set.insert(name);
                    }
                }
            }
        }

        *cache_guard = Some(PathCache {
            binaries: set.clone(),
            last_updated: Instant::now(),
        });

        set
    } else {
        HashSet::new()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandStatus {
    Valid,   // Exact match (Green)
    Prefix,  // Prefix of a valid command (White)
    Invalid, // No command matches (Red)
}

pub fn get_command_status(cmd: &str) -> CommandStatus {
    if cmd.is_empty() {
        return CommandStatus::Prefix;
    }

    let lower = cmd.to_lowercase();

    let builtins = [
        "cd",
        "pwd",
        "ls",
        "dir",
        "cat",
        "type",
        "edit",
        "ledit",
        "touch",
        "mkdir",
        "rm",
        "del",
        "which",
        "where",
        "clear",
        "cls",
        "history",
        "help",
        "exit",
        "quit",
        "export",
        "secret",
        "..",
        "...",
        "....",
        "tree",
        "sys",
        "info",
        "update",
        "updater",
        "install-wt",
        "wt-install",
        "install-vscode",
        "vscode-install",
        "install",
        "z",
        "jump",
        "alias",
        "search",
        "find",
        "usage",
        "du",
        "bench",
        "time",
        "top",
        "ps",
        "version",
        "head",
        "tail",
        "cp",
        "mv",
        "theme",
    ];

    if builtins.contains(&lower.as_str()) {
        return CommandStatus::Valid;
    }

    let path_binaries = get_path_binaries();
    if path_binaries.contains(&lower) {
        return CommandStatus::Valid;
    }

    let path = std::path::Path::new(cmd);
    if path.exists() && path.is_file() {
        return CommandStatus::Valid;
    }

    for b in builtins {
        if b.starts_with(&lower) {
            return CommandStatus::Prefix;
        }
    }

    for bin in &path_binaries {
        if bin.starts_with(&lower) {
            return CommandStatus::Prefix;
        }
    }

    CommandStatus::Invalid
}

fn autocomplete_command_or_file(input: &str) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    let trimmed = input.trim_start();
    let ends_with_space = input.ends_with(' ');
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    if parts.len() == 1 && !ends_with_space {
        let query = parts[0].to_lowercase();
        let builtins = [
            "cd",
            "pwd",
            "ls",
            "dir",
            "cat",
            "type",
            "edit",
            "ledit",
            "touch",
            "mkdir",
            "rm",
            "del",
            "which",
            "where",
            "clear",
            "cls",
            "history",
            "help",
            "exit",
            "quit",
            "export",
            "tree",
            "sys",
            "info",
            "update",
            "updater",
            "install-wt",
            "install-vscode",
            "install",
            "cargo",
            "git",
            "npm",
            "npx",
            "node",
            "python",
            "docker",
            "z",
            "jump",
            "alias",
            "search",
            "find",
            "usage",
            "du",
            "bench",
            "time",
            "top",
            "ps",
            "version",
            "head",
            "tail",
            "cp",
            "mv",
            "theme",
        ];
        for b in builtins {
            if b.starts_with(&query) && b != query {
                return Some(b.to_string());
            }
        }

        let path_binaries = get_path_binaries();
        let mut matching_bins: Vec<_> = path_binaries
            .iter()
            .filter(|bin| bin.starts_with(&query) && *bin != &query)
            .collect();
        matching_bins.sort();
        if let Some(first_match) = matching_bins.first() {
            return Some(first_match.to_string());
        }
    }

    if (parts.len() == 2 && !ends_with_space) || (parts.len() == 1 && ends_with_space) {
        let cmd = parts[0].to_lowercase();
        let subquery = if parts.len() == 2 && !ends_with_space {
            parts[1].to_lowercase()
        } else {
            String::new()
        };

        let subcommands: &[&str] = match cmd.as_str() {
            "git" => &[
                "status", "commit", "push", "pull", "checkout", "branch", "diff", "log", "add",
                "clone", "merge", "rebase",
            ],
            "cargo" => &[
                "build", "check", "test", "run", "clippy", "fmt", "add", "update", "clean",
            ],
            "npm" => &["start", "run", "test", "install", "build", "dev"],
            "docker" => &["ps", "run", "build", "exec", "stop", "images", "compose"],
            _ => &[],
        };

        for sub in subcommands {
            if sub.starts_with(&subquery) && *sub != subquery {
                return Some(format!("{} {}", parts[0], sub));
            }
        }
    }

    let raw_target = if ends_with_space {
        ""
    } else {
        parts.last().copied().unwrap_or("")
    };

    let (dir_prefix, file_query) = if let Some(idx) = raw_target.rfind('/') {
        (&raw_target[..=idx], &raw_target[idx + 1..])
    } else if let Some(idx) = raw_target.rfind('\\') {
        (&raw_target[..=idx], &raw_target[idx + 1..])
    } else {
        ("", raw_target)
    };

    let target_dir = if dir_prefix.is_empty() {
        std::env::current_dir().ok()?
    } else {
        std::env::current_dir().ok()?.join(dir_prefix)
    };

    if let Ok(entries) = std::fs::read_dir(target_dir) {
        let mut matches = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_lowercase().starts_with(&file_query.to_lowercase()) && name != file_query {
                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let completed_name = if is_dir { format!("{}/", name) } else { name };
                matches.push(completed_name);
            }
        }

        if let Some(matched) = matches.first() {
            let full_completed_target = format!("{}{}", dir_prefix, matched);
            if ends_with_space {
                return Some(format!("{} {}", input.trim_end(), full_completed_target));
            } else {
                let prefix_part = if parts.len() > 1 {
                    format!("{} ", parts[..parts.len() - 1].join(" "))
                } else {
                    String::new()
                };
                return Some(format!("{}{}", prefix_part, full_completed_target));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autocomplete_command_name() {
        assert_eq!(
            autocomplete_command_or_file("car"),
            Some("cargo".to_string())
        );
        assert_eq!(
            autocomplete_command_or_file("searc"),
            Some("search".to_string())
        );
        assert_eq!(
            autocomplete_command_or_file("upda"),
            Some("update".to_string())
        );
    }

    #[test]
    fn test_autocomplete_subcommand() {
        assert_eq!(
            autocomplete_command_or_file("git st"),
            Some("git status".to_string())
        );
        assert_eq!(
            autocomplete_command_or_file("cargo ch"),
            Some("cargo check".to_string())
        );
        assert_eq!(
            autocomplete_command_or_file("npm st"),
            Some("npm start".to_string())
        );
    }

    #[test]
    fn test_unicode_char_boundary_insert() {
        let mut buffer = String::new();
        let mut cursor_pos = 0;

        let accent = '´'; // 2-byte UTF-8 char
        buffer.insert(cursor_pos, accent);
        cursor_pos += accent.len_utf8();

        assert_eq!(cursor_pos, 2);
        assert!(buffer.is_char_boundary(cursor_pos));

        let ch_p = 'p';
        buffer.insert(cursor_pos, ch_p);
        cursor_pos += ch_p.len_utf8();
        assert_eq!(buffer, "´p");
        assert_eq!(cursor_pos, 3);
    }

    #[test]
    fn test_command_status_highlighting() {
        assert_eq!(get_command_status("cargo"), CommandStatus::Valid);
        assert_eq!(get_command_status("carg"), CommandStatus::Prefix);
        assert_eq!(get_command_status("car"), CommandStatus::Prefix);
        assert_eq!(
            get_command_status("carggo_invalid_xyz"),
            CommandStatus::Invalid
        );
    }

    #[test]
    fn test_autosuggestion_filters_out_typos() {
        let history = vec![
            "cargo run -- --cli".to_string(),
            "caro run".to_string(),
            "carggo run".to_string(),
        ];
        let buffer = "car".to_string();

        let suggest = history
            .iter()
            .rev()
            .find(|cmd| {
                if !cmd.to_lowercase().starts_with(&buffer.to_lowercase()) || *cmd == &buffer {
                    return false;
                }
                let first_word = cmd.split_whitespace().next().unwrap_or("");
                get_command_status(first_word) == CommandStatus::Valid
            })
            .map(|cmd| cmd[buffer.len()..].to_string());

        assert_eq!(suggest, Some("go run -- --cli".to_string()));
    }
}
