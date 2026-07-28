use crossterm::{
    cursor::{Hide, MoveToColumn, MoveUp, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
};
use std::io::{stdout, Write};

pub fn input_text(
    prompt: &str,
    default_val: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = stdout();

    let display_prompt = if let Some(def) = default_val {
        format!("{} [default: {}]: ", prompt, def)
    } else {
        format!("{}: ", prompt)
    };

    queue!(
        stdout,
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(75)),
        Print(&display_prompt),
        ResetColor
    )?;
    stdout.flush()?;

    let mut buf = String::new();
    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Enter => {
                    execute!(stdout, Print("\r\n"))?;
                    disable_raw_mode()?;
                    if buf.trim().is_empty() {
                        if let Some(def) = default_val {
                            return Ok(def.to_string());
                        }
                    }
                    return Ok(buf);
                }
                KeyCode::Char(c) => {
                    buf.push(c);
                    queue!(stdout, Print(c))?;
                    stdout.flush()?;
                }
                KeyCode::Backspace
                    if !buf.is_empty() => {
                        buf.pop();
                        queue!(stdout, MoveToColumn(0), Clear(ClearType::UntilNewLine))?;
                        queue!(
                            stdout,
                            SetAttribute(Attribute::Bold),
                            SetForegroundColor(Color::AnsiValue(75)),
                            Print(&display_prompt),
                            ResetColor,
                            Print(&buf)
                        )?;
                        stdout.flush()?;
                    }
                KeyCode::Esc => {
                    execute!(stdout, Print("\r\n"))?;
                    disable_raw_mode()?;
                    if let Some(def) = default_val {
                        return Ok(def.to_string());
                    }
                    return Ok(buf);
                }
                _ => {}
            }
        }
    }
}

pub fn choice_single(
    title: &str,
    options: &[String],
    default_index: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
    if options.is_empty() {
        return Ok(0);
    }
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide)?;

    let mut selected = if default_index < options.len() {
        default_index
    } else {
        0
    };

    println!(
        "\r\n {}{}? {}{}  (Use ↑/↓ keys, Enter to select)",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(220)),
        title,
        ResetColor
    );

    let option_count = options.len();
    render_single_options(&mut stdout, options, selected, true)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if selected > 0 {
                        selected -= 1;
                    } else {
                        selected = option_count - 1;
                    }
                    render_single_options(&mut stdout, options, selected, false)?;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if selected + 1 < option_count {
                        selected += 1;
                    } else {
                        selected = 0;
                    }
                    render_single_options(&mut stdout, options, selected, false)?;
                }
                KeyCode::Enter => {
                    execute!(stdout, Show)?;
                    execute!(stdout, Print("\r\n"))?;
                    disable_raw_mode()?;
                    return Ok(selected);
                }
                KeyCode::Esc => {
                    execute!(stdout, Show)?;
                    execute!(stdout, Print("\r\n"))?;
                    disable_raw_mode()?;
                    return Ok(selected);
                }
                _ => {}
            }
        }
    }
}

fn render_single_options(
    stdout: &mut std::io::Stdout,
    options: &[String],
    selected: usize,
    first_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !first_render {
        queue!(stdout, MoveUp(options.len() as u16))?;
    }

    for (i, opt) in options.iter().enumerate() {
        queue!(stdout, MoveToColumn(0), Clear(ClearType::UntilNewLine))?;
        if i == selected {
            queue!(
                stdout,
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(78)),
                Print(format!("  ❯ {}\r\n", opt)),
                ResetColor
            )?;
        } else {
            queue!(
                stdout,
                SetForegroundColor(Color::AnsiValue(245)),
                Print(format!("    {}\r\n", opt)),
                ResetColor
            )?;
        }
    }
    stdout.flush()?;
    Ok(())
}

pub fn choice_multi(
    title: &str,
    options: &[String],
    default_selected: &[usize],
) -> Result<Vec<usize>, Box<dyn std::error::Error>> {
    if options.is_empty() {
        return Ok(Vec::new());
    }
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, Hide)?;

    let mut cursor = 0;
    let mut selected_flags = vec![false; options.len()];
    for &idx in default_selected {
        if idx < selected_flags.len() {
            selected_flags[idx] = true;
        }
    }

    println!(
        "\r\n {}{}? {}{}  (Use ↑/↓ move, Space toggle, Enter confirm)",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(220)),
        title,
        ResetColor
    );

    let option_count = options.len();
    render_multi_options(&mut stdout, options, &selected_flags, cursor, true)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Release {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if cursor > 0 {
                        cursor -= 1;
                    } else {
                        cursor = option_count - 1;
                    }
                    render_multi_options(&mut stdout, options, &selected_flags, cursor, false)?;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if cursor + 1 < option_count {
                        cursor += 1;
                    } else {
                        cursor = 0;
                    }
                    render_multi_options(&mut stdout, options, &selected_flags, cursor, false)?;
                }
                KeyCode::Char(' ') => {
                    selected_flags[cursor] = !selected_flags[cursor];
                    render_multi_options(&mut stdout, options, &selected_flags, cursor, false)?;
                }
                KeyCode::Char('a') => {
                    let all_selected = selected_flags.iter().all(|&b| b);
                    for f in selected_flags.iter_mut() {
                        *f = !all_selected;
                    }
                    render_multi_options(&mut stdout, options, &selected_flags, cursor, false)?;
                }
                KeyCode::Enter | KeyCode::Esc => {
                    execute!(stdout, Show)?;
                    execute!(stdout, Print("\r\n"))?;
                    disable_raw_mode()?;
                    let result: Vec<usize> = selected_flags
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &sel)| if sel { Some(i) } else { None })
                        .collect();
                    return Ok(result);
                }
                _ => {}
            }
        }
    }
}

fn render_multi_options(
    stdout: &mut std::io::Stdout,
    options: &[String],
    selected_flags: &[bool],
    cursor: usize,
    first_render: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if !first_render {
        queue!(stdout, MoveUp(options.len() as u16))?;
    }

    for (i, opt) in options.iter().enumerate() {
        queue!(stdout, MoveToColumn(0), Clear(ClearType::UntilNewLine))?;
        let check_mark = if selected_flags[i] { "[x]" } else { "[ ]" };
        if i == cursor {
            queue!(
                stdout,
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(78)),
                Print(format!("  ❯ {} {}\r\n", check_mark, opt)),
                ResetColor
            )?;
        } else {
            let color = if selected_flags[i] {
                Color::AnsiValue(114)
            } else {
                Color::AnsiValue(245)
            };
            queue!(
                stdout,
                SetForegroundColor(color),
                Print(format!("    {} {}\r\n", check_mark, opt)),
                ResetColor
            )?;
        }
    }
    stdout.flush()?;
    Ok(())
}
