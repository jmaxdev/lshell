use crossterm::style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor};
use std::fmt::Write;

#[derive(Debug, Clone)]
pub struct Segment {
    pub text: String,
    pub fg: Color,
    pub bg: Color,
}

impl Segment {
    pub fn new(text: impl Into<String>, fg: Color, bg: Color) -> Self {
        Self {
            text: text.into(),
            fg,
            bg,
        }
    }
}

pub struct PowerlineRenderer {
    pub use_powerline_symbols: bool,
}

impl PowerlineRenderer {
    pub fn new(use_powerline_symbols: bool) -> Self {
        Self { use_powerline_symbols }
    }

    pub fn render(&self, segments: &[Segment]) -> String {
        let mut buffer = String::new();

        if segments.is_empty() {
            return buffer;
        }

        let arrow_right = if self.use_powerline_symbols {
            ""
        } else {
            "►"
        };

        for i in 0..segments.len() {
            let seg = &segments[i];

            let _ = write!(
                buffer,
                "{}{}{}",
                SetBackgroundColor(seg.bg),
                SetForegroundColor(seg.fg),
                seg.text
            );

            if i + 1 < segments.len() {
                let next_seg = &segments[i + 1];
                let _ = write!(
                    buffer,
                    "{}{}{}",
                    SetBackgroundColor(next_seg.bg),
                    SetForegroundColor(seg.bg),
                    arrow_right
                );
            } else {
                let _ = write!(
                    buffer,
                    "{}{}{}",
                    ResetColor,
                    SetForegroundColor(seg.bg),
                    arrow_right
                );
            }
        }

        let _ = write!(buffer, "{}", ResetColor);
        buffer
    }
}
