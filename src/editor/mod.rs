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
            let autosuggestion = if config.enable_autosuggestions
                && buffer.len() >= config.min_autosuggestion_len
                && !search_mode
                && cursor_pos == buffer.len()
            {
                history
                    .iter()
                    .rev()
                    .find(|cmd| {
                        if !cmd.starts_with(&buffer) || *cmd == &buffer {
                            return false;
                        }
                        let first_word = cmd.split_whitespace().next().unwrap_or("");
                        is_valid_cmd(first_word)
                    })
                    .map(|cmd| cmd[buffer.len()..].to_string())
            } else {
                None
            };

            if let Event::Key(key_event) = event::read()? {
                if key_event.kind == KeyEventKind::Release {
                    continue;
                }

                let KeyEvent { code, modifiers, .. } = key_event;

                if code == KeyCode::Char('r') && modifiers.contains(KeyModifiers::CONTROL) {
                    search_mode = !search_mode;
                    search_query.clear();
                }

                if search_mode {
                    match code {
                        KeyCode::Esc | KeyCode::Enter => {
                            if !search_query.is_empty() {
                                if let Some(found) = history.iter().rev().find(|c| c.contains(&search_query)) {
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

                        KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                            if buffer.is_empty() {
                                println!("exit");
                                disable_raw_mode()?;
                                return Ok("exit".to_string());
                            }
                        }

                        KeyCode::Right => {
                            if cursor_pos < buffer.len() {
                                cursor_pos += 1;
                            } else if let Some(ref suggest) = autosuggestion {
                                buffer.push_str(suggest);
                                cursor_pos = buffer.len();
                            }
                        }

                        KeyCode::Left => {
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                            }
                        }

                        KeyCode::Backspace => {
                            if cursor_pos > 0 {
                                cursor_pos -= 1;
                                buffer.remove(cursor_pos);
                            }
                        }

                        KeyCode::Delete => {
                            if cursor_pos < buffer.len() {
                                buffer.remove(cursor_pos);
                            }
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

                        KeyCode::Up => {
                            if !history.is_empty() && history_index > 0 {
                                history_index -= 1;
                                buffer = history[history_index].clone();
                                cursor_pos = buffer.len();
                            }
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
                            if let Some(ref suggest) = autosuggestion {
                                buffer.push_str(suggest);
                                cursor_pos = buffer.len();
                            } else if let Some(completed) = autocomplete_file(&buffer) {
                                buffer = completed;
                                cursor_pos = buffer.len();
                            }
                        }

                        KeyCode::Char(c) => {
                            buffer.insert(cursor_pos, c);
                            cursor_pos += 1;
                        }

                        _ => {}
                    }
                }

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
                        let is_valid = is_valid_cmd(cmd_part);
                        let cmd_color = if is_valid {
                            Color::AnsiValue(78)
                        } else if !cmd_part.is_empty() {
                            Color::AnsiValue(203)
                        } else {
                            Color::AnsiValue(255)
                        };

                        queue!(stdout, SetForegroundColor(cmd_color), Print(cmd_part), ResetColor)?;
                    } else {
                        queue!(stdout, SetForegroundColor(Color::AnsiValue(255)), Print(cmd_part), ResetColor)?;
                    }

                    queue!(stdout, SetForegroundColor(Color::AnsiValue(252)), Print(rest_part), ResetColor)?;

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
                let target_col = prompt_prefix_len + cursor_pos;
                queue!(stdout, MoveToColumn(target_col as u16))?;
                stdout.flush()?;
            }
        }
    }
}

fn is_valid_cmd(cmd: &str) -> bool {
    if cmd.is_empty() {
        return true;
    }
    let lower = cmd.to_lowercase();
    let builtins = [
        "cd", "pwd", "ls", "dir", "cat", "type", "edit", "ledit", "touch",
        "mkdir", "rm", "del", "which", "where", "clear", "cls", "history", "help", "exit", "quit",
        "export", "..", "...", "....", "tree", "sys", "info", "update", "updater", "install-wt", "wt-install", "install-vscode", "vscode-install", "install", "cargo", "git", "npm", "npx", "node",
        "python", "py", "rustc", "code", "cmd", "powershell", "curl", "wget", "ssh", "docker",
        "z", "jump", "alias", "search", "find", "usage", "du", "bench", "time", "top", "ps",
    ];
    builtins.contains(&lower.as_str())
}

fn autocomplete_file(input: &str) -> Option<String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let query = parts.last().copied().unwrap_or("");
    let current_dir = std::env::current_dir().ok()?;

    if let Ok(entries) = std::fs::read_dir(current_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.to_lowercase().starts_with(&query.to_lowercase()) && name != query {
                if parts.len() > 1 {
                    let mut prefix = parts[..parts.len() - 1].join(" ");
                    prefix.push(' ');
                    prefix.push_str(&name);
                    return Some(prefix);
                } else {
                    return Some(name);
                }
            }
        }
    }
    None
}
