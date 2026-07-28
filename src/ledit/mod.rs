use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, Clear, ClearType},
};
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;

pub struct LeditEditor;

impl LeditEditor {
    pub fn open(filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from(filename);
        let mut is_new_file = false;
        let mut lines: Vec<String> = if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => content.lines().map(|s| s.to_string()).collect(),
                Err(_) => {
                    is_new_file = true;
                    vec![String::new()]
                }
            }
        } else {
            is_new_file = true;
            vec![String::new()]
        };

        if lines.is_empty() {
            lines.push(String::new());
        }

        let initial_line_count = lines.len();
        let mut stdout = stdout();
        
        execute!(stdout, EnterAlternateScreen)?;
        enable_raw_mode()?;

        let mut cx: usize = 0;
        let mut cy: usize = 0;
        let mut row_offset: usize = 0;
        let mut modified = false;
        let mut clipboard = String::new();
        let mut prompt_exit_confirm = false;

        let mut find_mode = false;
        let mut find_query = String::new();

        let mut status_message = if is_new_file {
            "[ New File ]".to_string()
        } else {
            format!("[ Read {} lines ]", initial_line_count)
        };

        loop {
            let (cols, rows) = crossterm::terminal::size().unwrap_or((80, 24));
            let max_content_rows = rows.saturating_sub(4) as usize;

            if cy < row_offset {
                row_offset = cy;
            }
            if cy >= row_offset + max_content_rows {
                row_offset = cy.saturating_sub(max_content_rows - 1);
            }

            if cy >= lines.len() {
                cy = lines.len().saturating_sub(1);
            }
            if cx > lines[cy].len() {
                cx = lines[cy].len();
            }

            queue!(stdout, Hide)?;

            let mod_tag = if modified { " Modified" } else { "" };
            let left_text = "  Little Edit 1.0";
            let center_text = format!("File: {}", filename);

            let header_bar = format!(
                "  {:<16} {:^center_width$} {:>10}  ",
                left_text,
                center_text,
                mod_tag,
                center_width = (cols as usize).saturating_sub(32)
            );

            queue!(
                stdout,
                MoveTo(0, 0),
                SetBackgroundColor(Color::AnsiValue(255)),
                SetForegroundColor(Color::AnsiValue(0)),
                SetAttribute(Attribute::Bold),
                Print(format!("{:<width$}", header_bar, width = cols as usize)),
                ResetColor
            )?;

            for row in 0..max_content_rows {
                let line_idx = row_offset + row;
                queue!(stdout, MoveTo(0, (row + 1) as u16), Clear(ClearType::UntilNewLine))?;
                if line_idx < lines.len() {
                    let line_str = &lines[line_idx];
                    let max_line_len = (cols as usize).saturating_sub(7);
                    let print_str = if line_str.len() > max_line_len {
                        &line_str[..max_line_len]
                    } else {
                        line_str
                    };
                    queue!(
                        stdout,
                        SetForegroundColor(Color::AnsiValue(242)),
                        Print(format!("{:4} │ ", line_idx + 1)),
                        ResetColor
                    )?;
                    render_syntax_highlighted_line(&mut stdout, print_str)?;
                }
            }

            let status_formatted = format!(" {:<width$}", status_message, width = (cols as usize).saturating_sub(1));
            queue!(
                stdout,
                MoveTo(0, rows - 3),
                SetForegroundColor(Color::AnsiValue(75)),
                SetAttribute(Attribute::Bold),
                Print(format!("{:<width$}", status_formatted, width = cols as usize)),
                ResetColor
            )?;

            draw_key_row(
                &mut stdout,
                rows - 2,
                cols as usize,
                &[
                    ("^G", "Get Help"),
                    ("^O", "WriteOut"),
                    ("^F", "Find Text"),
                    ("^K", "Cut Line"),
                    ("^J", "Justify"),
                    ("^C", "Location"),
                ],
            )?;

            draw_key_row(
                &mut stdout,
                rows - 1,
                cols as usize,
                &[
                    ("^X", "Exit"),
                    ("^R", "Read File"),
                    ("^\\", "Replace"),
                    ("^U", "Paste"),
                    ("^T", "Execute"),
                    ("^_", "Go To Line"),
                ],
            )?;

            let screen_row = 1 + (cy - row_offset);
            let screen_col = 7 + cx;
            queue!(stdout, MoveTo(screen_col as u16, screen_row as u16), Show)?;

            stdout.flush()?;

            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }

                let KeyEvent { code, modifiers, .. } = key;

                if find_mode {
                    match code {
                        KeyCode::Esc => {
                            find_mode = false;
                            status_message = "[ Cancelled search ]".to_string();
                        }
                        KeyCode::Enter => {
                            find_mode = false;
                            if !find_query.is_empty() {
                                let mut found = false;
                                for i in 0..lines.len() {
                                    let line_idx = (cy + i) % lines.len();
                                    if let Some(col_idx) = lines[line_idx].find(&find_query) {
                                        if line_idx != cy || col_idx > cx || i > 0 {
                                            cy = line_idx;
                                            cx = col_idx;
                                            status_message = format!("[ Match found at line {}, col {} ]", cy + 1, cx + 1);
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                                if !found {
                                    status_message = format!("[ '{}' not found ]", find_query);
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            find_query.pop();
                            status_message = format!(" Search: {} ", find_query);
                        }
                        KeyCode::Char(c) => {
                            find_query.push(c);
                            status_message = format!(" Search: {} ", find_query);
                        }
                        _ => {}
                    }
                    continue;
                }

                if (code == KeyCode::Char('f') || code == KeyCode::Char('w')) && modifiers.contains(KeyModifiers::CONTROL) {
                    find_mode = true;
                    find_query.clear();
                    status_message = " Search: ".to_string();
                    continue;
                }

                if prompt_exit_confirm {
                    prompt_exit_confirm = false;
                    if code == KeyCode::Char('y') || code == KeyCode::Char('Y') {
                        let content = lines.join("\n");
                        let _ = fs::write(&path, content);
                        break;
                    } else if code == KeyCode::Char('n') || code == KeyCode::Char('N') {
                        break;
                    } else {
                        status_message = "[ Cancelled ]".to_string();
                        continue;
                    }
                }

                if code == KeyCode::Char('x') && modifiers.contains(KeyModifiers::CONTROL) {
                    if modified {
                        status_message = " Save modified buffer? (Answering 'Y' saves, 'N' exits without saving) ".to_string();
                        prompt_exit_confirm = true;
                        continue;
                    } else {
                        break;
                    }
                }

                if (code == KeyCode::Char('o') || code == KeyCode::Char('s')) && modifiers.contains(KeyModifiers::CONTROL) {
                    let content = lines.join("\n");
                    match fs::write(&path, content) {
                        Ok(_) => {
                            modified = false;
                            status_message = format!("[ Wrote {} lines ]", lines.len());
                        }
                        Err(e) => status_message = format!("[ Error writing: {} ]", e),
                    }
                    continue;
                }

                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    let total_chars: usize = lines.iter().map(|l| l.len()).sum();
                    status_message = format!(
                        "[ line {}/{} ({:.0}%), col {}/{}, char {}/{} ]",
                        cy + 1,
                        lines.len(),
                        ((cy + 1) as f64 / lines.len() as f64) * 100.0,
                        cx + 1,
                        lines[cy].len() + 1,
                        cx + 1,
                        total_chars
                    );
                    continue;
                }

                if code == KeyCode::Char('k') && modifiers.contains(KeyModifiers::CONTROL) {
                    clipboard = lines.remove(cy);
                    if lines.is_empty() {
                        lines.push(String::new());
                    }
                    if cy >= lines.len() {
                        cy = lines.len() - 1;
                    }
                    cx = 0;
                    modified = true;
                    status_message = "[ Cut 1 line ]".to_string();
                    continue;
                }

                if code == KeyCode::Char('u') && modifiers.contains(KeyModifiers::CONTROL) {
                    if !clipboard.is_empty() {
                        lines.insert(cy, clipboard.clone());
                        modified = true;
                        status_message = "[ Pasted 1 line ]".to_string();
                    }
                    continue;
                }

                match code {
                    KeyCode::Up => {
                        if cy > 0 {
                            cy -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if cy + 1 < lines.len() {
                            cy += 1;
                        }
                    }
                    KeyCode::Left => {
                        if cx > 0 {
                            cx -= 1;
                        } else if cy > 0 {
                            cy -= 1;
                            cx = lines[cy].len();
                        }
                    }
                    KeyCode::Right => {
                        if cx < lines[cy].len() {
                            cx += 1;
                        } else if cy + 1 < lines.len() {
                            cy += 1;
                            cx = 0;
                        }
                    }
                    KeyCode::Backspace => {
                        if cx > 0 {
                            lines[cy].remove(cx - 1);
                            cx -= 1;
                            modified = true;
                        } else if cy > 0 {
                            let prev_len = lines[cy - 1].len();
                            let current_line = lines.remove(cy);
                            cy -= 1;
                            lines[cy].push_str(&current_line);
                            cx = prev_len;
                            modified = true;
                        }
                    }
                    KeyCode::Enter => {
                        let remainder = lines[cy].split_off(cx);
                        lines.insert(cy + 1, remainder);
                        cy += 1;
                        cx = 0;
                        modified = true;
                    }
                    KeyCode::Char(c) => {
                        lines[cy].insert(cx, c);
                        cx += 1;
                        modified = true;
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(stdout, Show, LeaveAlternateScreen)?;
        Ok(())
    }
}

fn render_syntax_highlighted_line(
    stdout: &mut std::io::Stdout,
    line: &str,
) -> Result<(), std::io::Error> {
    let trimmed = line.trim();
    if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("<!--") {
        queue!(stdout, SetForegroundColor(Color::AnsiValue(243)), Print(line), ResetColor)?;
        return Ok(());
    }

    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        queue!(
            stdout,
            SetForegroundColor(Color::AnsiValue(214)),
            SetAttribute(Attribute::Bold),
            Print(line),
            ResetColor
        )?;
        return Ok(());
    }

    let keywords = [
        // Rust
        "fn", "pub", "let", "mut", "struct", "impl", "use", "match", "return", "if", "else",
        "true", "false", "type", "mod", "crate", "enum", "trait", "where", "async", "await",
        "loop", "while", "for", "in", "break", "continue", "self", "Self", "dyn", "ref", "static",
        // Python
        "def", "class", "import", "from", "as", "try", "except", "finally", "with", "raise",
        "lambda", "is", "not", "and", "or", "None", "pass", "yield", "global", "nonlocal",
        // JavaScript / TypeScript
        "function", "const", "var", "interface", "export", "default", "extends", "implements",
        "constructor", "typeof", "instanceof", "new", "delete", "void", "any", "string", "number",
        "boolean", "null", "undefined",
        // C / C++
        "int", "char", "float", "double", "unsigned", "signed", "long", "short", "include",
        "define", "namespace", "template", "typename", "public", "private", "protected",
        // Configuration / Declarative
        "package", "version", "dependencies", "description", "authors", "license", "scripts",
    ];

    let words = line.split_inclusive(|c: char| !c.is_alphanumeric() && c != '_');
    let mut in_string = false;

    for word in words {
        if word.contains('"') || word.contains('\'') || word.contains('`') {
            in_string = !in_string;
            queue!(stdout, SetForegroundColor(Color::AnsiValue(220)), Print(word), ResetColor)?;
            continue;
        }

        if in_string {
            queue!(stdout, SetForegroundColor(Color::AnsiValue(220)), Print(word), ResetColor)?;
            continue;
        }

        let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '_');
        if keywords.contains(&clean) {
            queue!(
                stdout,
                SetForegroundColor(Color::AnsiValue(75)),
                SetAttribute(Attribute::Bold),
                Print(word),
                ResetColor
            )?;
        } else if !clean.is_empty() && clean.chars().all(|c| c.is_ascii_digit()) {
            queue!(stdout, SetForegroundColor(Color::AnsiValue(81)), Print(word), ResetColor)?;
        } else {
            queue!(stdout, SetForegroundColor(Color::AnsiValue(252)), Print(word), ResetColor)?;
        }
    }

    Ok(())
}

fn draw_key_row(
    stdout: &mut std::io::Stdout,
    row: u16,
    cols: usize,
    items: &[(&str, &str)],
) -> Result<(), std::io::Error> {
    queue!(stdout, MoveTo(0, row))?;

    let item_width = cols / items.len();

    for (idx, (key, label)) in items.iter().enumerate() {
        let is_last = idx == items.len() - 1;
        let pad_width = if is_last {
            cols.saturating_sub(idx * item_width + key.len() + 1)
        } else {
            item_width.saturating_sub(key.len() + 1)
        };

        queue!(
            stdout,
            SetBackgroundColor(Color::AnsiValue(255)),
            SetForegroundColor(Color::AnsiValue(0)),
            SetAttribute(Attribute::Bold),
            Print(format!("{}", key)),
            ResetColor,
            SetForegroundColor(Color::AnsiValue(250)),
            Print(format!(" {:<width$}", label, width = pad_width)),
            ResetColor
        )?;
    }

    Ok(())
}
