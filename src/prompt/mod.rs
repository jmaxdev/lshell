use crate::config::Config;
use crate::git::GitInfo;
use crate::theme::{PowerlineRenderer, Segment};
use crossterm::style::{Color, ResetColor, SetForegroundColor};
use std::env;
use std::fmt::Write;
use std::path::Path;

pub struct PromptBuilder;

impl PromptBuilder {
    pub fn build(last_exit_code: i32, config: &Config) -> String {
        match config.theme.as_str() {
            "agnoster" => Self::build_agnoster(last_exit_code, config),
            "nord" => Self::build_nord(last_exit_code, config),
            "dracula" => Self::build_dracula(last_exit_code, config),
            _ => Self::build_minimal(last_exit_code, config),
        }
    }

    fn build_minimal(last_exit_code: i32, config: &Config) -> String {
        let mut out = String::new();

        if last_exit_code == 0 {
            let _ = write!(
                out,
                "{}{}",
                SetForegroundColor(Color::AnsiValue(78)),
                config.success_symbol
            );
        } else {
            let _ = write!(
                out,
                "{}{}",
                SetForegroundColor(Color::AnsiValue(203)),
                config.error_symbol
            );
        }

        let _ = write!(
            out,
            "{}\u{2794} ",
            SetForegroundColor(Color::AnsiValue(245))
        );

        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let folder_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("~");

        let _ = write!(
            out,
            "{}{} ",
            SetForegroundColor(Color::AnsiValue(75)),
            folder_name
        );

        if config.show_dev_badge {
            if let Some((badge_icon, badge_color)) = detect_dev_badge(&current_dir) {
                let _ = write!(
                    out,
                    "{}[{}] ",
                    SetForegroundColor(Color::AnsiValue(badge_color)),
                    badge_icon
                );
            }
        }

        if config.show_git {
            if let Some(git) = GitInfo::get(&current_dir) {
                let dirty_flag = if git.is_dirty { "*" } else { "" };
                let _ = write!(
                    out,
                    "{}(\u{F418} {}{})",
                    SetForegroundColor(Color::AnsiValue(210)),
                    git.branch,
                    dirty_flag
                );
            }
        }

        let _ = write!(out, "{}\n", ResetColor);

        let _ = write!(
            out,
            "{}{}",
            SetForegroundColor(Color::AnsiValue(78)),
            config.prompt_symbol
        );

        out
    }

    fn build_nord(last_exit_code: i32, config: &Config) -> String {
        let mut segments = Vec::new();
        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let formatted_path = format_path(&current_dir);

        segments.push(Segment::new(
            format!("  {} ", formatted_path),
            Color::AnsiValue(15),
            Color::AnsiValue(31),
        ));

        if config.show_dev_badge {
            if let Some((badge_icon, _)) = detect_dev_badge(&current_dir) {
                segments.push(Segment::new(
                    format!(" {} ", badge_icon),
                    Color::AnsiValue(236),
                    Color::AnsiValue(110),
                ));
            }
        }

        if config.show_git {
            if let Some(git) = GitInfo::get(&current_dir) {
                let dirty_flag = if git.is_dirty { "*" } else { "" };
                segments.push(Segment::new(
                    format!("  {}{} ", git.branch, dirty_flag),
                    Color::AnsiValue(236),
                    Color::AnsiValue(150),
                ));
            }
        }

        if last_exit_code != 0 {
            segments.push(Segment::new(
                format!(" ✘ {} ", last_exit_code),
                Color::AnsiValue(15),
                Color::AnsiValue(167),
            ));
        }

        let renderer = PowerlineRenderer::new(config.use_powerline_symbols);
        let mut prompt = renderer.render(&segments);
        prompt.push_str("\n");
        prompt.push_str(&config.prompt_symbol);
        prompt
    }

    fn build_dracula(last_exit_code: i32, config: &Config) -> String {
        let mut segments = Vec::new();
        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let formatted_path = format_path(&current_dir);

        segments.push(Segment::new(
            format!("  {} ", formatted_path),
            Color::AnsiValue(15),
            Color::AnsiValue(141),
        ));

        if config.show_dev_badge {
            if let Some((badge_icon, _)) = detect_dev_badge(&current_dir) {
                segments.push(Segment::new(
                    format!(" {} ", badge_icon),
                    Color::AnsiValue(16),
                    Color::AnsiValue(213),
                ));
            }
        }

        if config.show_git {
            if let Some(git) = GitInfo::get(&current_dir) {
                let dirty_flag = if git.is_dirty { "*" } else { "" };
                segments.push(Segment::new(
                    format!("  {}{} ", git.branch, dirty_flag),
                    Color::AnsiValue(16),
                    Color::AnsiValue(84),
                ));
            }
        }

        if last_exit_code != 0 {
            segments.push(Segment::new(
                format!(" ✘ {} ", last_exit_code),
                Color::AnsiValue(15),
                Color::AnsiValue(203),
            ));
        }

        let renderer = PowerlineRenderer::new(config.use_powerline_symbols);
        let mut prompt = renderer.render(&segments);
        prompt.push_str("\n");
        prompt.push_str(&config.prompt_symbol);
        prompt
    }

    fn build_agnoster(last_exit_code: i32, config: &Config) -> String {
        let mut segments = Vec::new();
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_else(|_| "lshell".to_string());

        segments.push(Segment::new(
            format!("  {} ", username),
            Color::AnsiValue(15),
            Color::AnsiValue(236),
        ));

        let current_dir = env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf());
        let formatted_path = format_path(&current_dir);
        segments.push(Segment::new(
            format!("  {} ", formatted_path),
            Color::AnsiValue(16),
            Color::AnsiValue(31),
        ));

        if config.show_dev_badge {
            if let Some((badge_icon, _)) = detect_dev_badge(&current_dir) {
                segments.push(Segment::new(
                    format!(" {} ", badge_icon),
                    Color::AnsiValue(16),
                    Color::AnsiValue(220),
                ));
            }
        }

        if config.show_git {
            if let Some(git) = GitInfo::get(&current_dir) {
                let dirty_flag = if git.is_dirty { "*" } else { "" };
                segments.push(Segment::new(
                    format!("  {}{} ", git.branch, dirty_flag),
                    Color::AnsiValue(16),
                    if git.is_dirty {
                        Color::AnsiValue(214)
                    } else {
                        Color::AnsiValue(78)
                    },
                ));
            }
        }

        if last_exit_code != 0 {
            segments.push(Segment::new(
                format!(" ✘ {} ", last_exit_code),
                Color::AnsiValue(15),
                Color::AnsiValue(160),
            ));
        }

        let renderer = PowerlineRenderer::new(config.use_powerline_symbols);
        let mut prompt = renderer.render(&segments);
        prompt.push_str("\n");
        prompt.push_str(&config.prompt_symbol);
        prompt
    }
}

fn detect_dev_badge(dir: &Path) -> Option<(&'static str, u8)> {
    if dir.join("Cargo.toml").exists() {
        Some(("🦀 Rust", 208))
    } else if dir.join("package.json").exists() {
        Some(("⬢ Node", 114))
    } else if dir.join("pyproject.toml").exists() || dir.join("requirements.txt").exists() {
        Some(("🐍 Python", 220))
    } else if dir.join("Dockerfile").exists() || dir.join("docker-compose.yml").exists() {
        Some(("🐳 Docker", 75))
    } else if dir.join("go.mod").exists() {
        Some(("🐹 Go", 81))
    } else {
        None
    }
}

fn format_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            if stripped.components().count() == 0 {
                return "~".to_string();
            } else {
                return format!("~\\{}", stripped.display());
            }
        }
    }
    path.display().to_string()
}
