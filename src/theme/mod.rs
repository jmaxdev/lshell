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
        Self {
            use_powerline_symbols,
        }
    }

    pub fn render(&self, segments: &[Segment]) -> String {
        self.render_styled(segments, "powerline")
    }

    pub fn render_styled(&self, segments: &[Segment], style_name: &str) -> String {
        let mut buffer = String::new();

        if segments.is_empty() {
            return buffer;
        }

        match style_name.to_lowercase().as_str() {
            "rounded" | "capsule" | "pill" => {
                let left_cap = if self.use_powerline_symbols {
                    ""
                } else {
                    "("
                };
                let right_cap = if self.use_powerline_symbols {
                    ""
                } else {
                    ")"
                };

                for i in 0..segments.len() {
                    let seg = &segments[i];
                    if i == 0 {
                        let _ = write!(
                            buffer,
                            "{}{}{}{}{}",
                            ResetColor,
                            SetForegroundColor(seg.bg),
                            left_cap,
                            SetBackgroundColor(seg.bg),
                            SetForegroundColor(seg.fg),
                        );
                    } else {
                        let _ = write!(
                            buffer,
                            "{}{}",
                            SetBackgroundColor(seg.bg),
                            SetForegroundColor(seg.fg),
                        );
                    }

                    let _ = write!(buffer, "{}", seg.text);

                    if i + 1 < segments.len() {
                        let next_seg = &segments[i + 1];
                        let _ = write!(
                            buffer,
                            "{}{}{}",
                            SetBackgroundColor(next_seg.bg),
                            SetForegroundColor(seg.bg),
                            right_cap
                        );
                    } else {
                        let _ = write!(
                            buffer,
                            "{}{}{}",
                            ResetColor,
                            SetForegroundColor(seg.bg),
                            right_cap
                        );
                    }
                }
            }
            "slanted" | "diagonal" => {
                let arrow_right = if self.use_powerline_symbols {
                    ""
                } else {
                    "\\"
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
            }
            "minimal" | "inline" => {
                for seg in segments {
                    let _ = write!(
                        buffer,
                        "{}{}{} ",
                        ResetColor,
                        SetForegroundColor(seg.bg),
                        seg.text.trim()
                    );
                }
            }
            "brackets" | "bracket" => {
                for seg in segments {
                    let _ = write!(
                        buffer,
                        "{}{}[{}{}{}] ",
                        ResetColor,
                        SetForegroundColor(Color::AnsiValue(245)),
                        SetForegroundColor(seg.bg),
                        seg.text.trim(),
                        SetForegroundColor(Color::AnsiValue(245))
                    );
                }
            }
            "pure" | "simple" => {
                for seg in segments {
                    let _ = write!(
                        buffer,
                        "{}{}{} ",
                        ResetColor,
                        SetForegroundColor(seg.bg),
                        seg.text.trim()
                    );
                }
            }
            _ => {
                // Default Powerline
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
            }
        }

        let _ = write!(buffer, "{}", ResetColor);
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_styles() {
        let renderer = PowerlineRenderer::new(true);
        let segments = vec![
            Segment::new(" test ", Color::AnsiValue(15), Color::AnsiValue(31)),
            Segment::new(" master ", Color::AnsiValue(16), Color::AnsiValue(84)),
        ];

        let rounded = renderer.render_styled(&segments, "rounded");
        assert!(rounded.contains(""));
        assert!(rounded.contains(""));

        let slanted = renderer.render_styled(&segments, "slanted");
        assert!(slanted.contains(""));

        let minimal = renderer.render_styled(&segments, "minimal");
        assert!(minimal.contains("test"));

        let brackets = renderer.render_styled(&segments, "brackets");
        assert!(brackets.contains("test"));
        assert!(brackets.contains("["));
        assert!(brackets.contains("]"));
    }
}
