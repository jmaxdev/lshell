use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_theme")]
    pub theme: String,

    #[serde(default = "default_prompt_symbol")]
    pub prompt_symbol: String,

    #[serde(default = "default_success_symbol")]
    pub success_symbol: String,

    #[serde(default = "default_error_symbol")]
    pub error_symbol: String,

    #[serde(default = "default_true")]
    pub show_git: bool,

    #[serde(default = "default_true")]
    pub show_time: bool,

    #[serde(default = "default_true")]
    pub use_powerline_symbols: bool,

    #[serde(default = "default_true")]
    pub enable_autosuggestions: bool,

    #[serde(default = "default_min_suggest_len")]
    pub min_autosuggestion_len: usize,

    #[serde(default = "default_true")]
    pub enable_syntax_highlighting: bool,

    #[serde(default = "default_true")]
    pub show_dev_badge: bool,

    #[serde(default = "default_tree_depth")]
    pub tree_max_depth: usize,

    #[serde(default)]
    pub aliases: HashMap<String, String>,

    #[serde(default)]
    pub env: HashMap<String, String>,
}

fn default_theme() -> String {
    "minimal".to_string()
}

fn default_prompt_symbol() -> String {
    "$ ".to_string()
}

fn default_success_symbol() -> String {
    "○".to_string()
}

fn default_error_symbol() -> String {
    "✖".to_string()
}

fn default_true() -> bool {
    true
}

fn default_min_suggest_len() -> usize {
    2
}

fn default_tree_depth() -> usize {
    3
}

impl Default for Config {
    fn default() -> Self {
        let mut aliases = HashMap::new();
        aliases.insert("ll".to_string(), "ls -la".to_string());
        aliases.insert("g".to_string(), "git".to_string());
        aliases.insert("cls".to_string(), "clear".to_string());
        aliases.insert("e".to_string(), "edit".to_string());

        Self {
            theme: default_theme(),
            prompt_symbol: default_prompt_symbol(),
            success_symbol: default_success_symbol(),
            error_symbol: default_error_symbol(),
            show_git: true,
            show_time: true,
            use_powerline_symbols: true,
            enable_autosuggestions: true,
            min_autosuggestion_len: 2,
            enable_syntax_highlighting: true,
            show_dev_badge: true,
            tree_max_depth: 3,
            aliases,
            env: HashMap::new(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".lshell");
            if config_path.exists() {
                if let Ok(content) = fs::read_to_string(&config_path) {
                    if let Ok(cfg) = toml::from_str::<Config>(&content) {
                        for (k, v) in &cfg.env {
                            let plain_val = crate::security::decrypt_val(v);
                            std::env::set_var(k, plain_val);
                        }
                        return cfg;
                    }
                }
            }
        }
        let cfg = Config::default();
        cfg.save();
        cfg
    }

    pub fn save(&self) {
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".lshell");
            if let Ok(content) = toml::to_string_pretty(self) {
                let _ = fs::write(config_path, content);
            }
        }
    }

    pub fn save_default() {
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".lshell");
            if !config_path.exists() {
                let default_cfg = Config::default();
                default_cfg.save();
            }
        }
    }
}
