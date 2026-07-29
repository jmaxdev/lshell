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

    #[serde(default)]
    pub custom_themes: HashMap<String, CustomTheme>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomTheme {
    pub name: String,
    #[serde(default = "default_style")]
    pub style: String,
    #[serde(default = "default_line_layout")]
    pub line_layout: String,
    pub path_fg: u8,
    pub path_bg: u8,
    pub git_fg: u8,
    pub git_bg: u8,
    pub badge_fg: u8,
    pub badge_bg: u8,
    pub user_fg: Option<u8>,
    pub user_bg: Option<u8>,
    #[serde(default)]
    pub host_fg: Option<u8>,
    #[serde(default)]
    pub host_bg: Option<u8>,
    #[serde(default)]
    pub time_fg: Option<u8>,
    #[serde(default)]
    pub time_bg: Option<u8>,
    #[serde(default)]
    pub error_fg: Option<u8>,
    #[serde(default)]
    pub error_bg: Option<u8>,
    #[serde(default)]
    pub prompt_symbol_fg: Option<u8>,
    #[serde(default)]
    pub show_dev_badge: Option<bool>,
    #[serde(default = "default_true")]
    pub use_powerline: bool,
    pub prompt_symbol: Option<String>,
}

fn default_style() -> String {
    "powerline".to_string()
}

fn default_line_layout() -> String {
    "double_line".to_string()
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
            custom_themes: HashMap::new(),
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

    pub fn load_local_aliases(dir: &std::path::Path) -> HashMap<String, String> {
        let local_path = dir.join(".lshell_alias");
        if local_path.exists() {
            if let Ok(content) = fs::read_to_string(&local_path) {
                if let Ok(map) = toml::from_str::<HashMap<String, String>>(&content) {
                    return map;
                }
            }
        }
        HashMap::new()
    }

    pub fn save_local_aliases(dir: &std::path::Path, aliases: &HashMap<String, String>) {
        let local_path = dir.join(".lshell_alias");
        if aliases.is_empty() {
            if local_path.exists() {
                let _ = fs::remove_file(local_path);
            }
        } else if let Ok(content) = toml::to_string_pretty(aliases) {
            let _ = fs::write(local_path, content);
        }
    }

    pub fn get_merged_aliases(&self, dir: &std::path::Path) -> HashMap<String, String> {
        let mut merged = self.aliases.clone();
        let local = Self::load_local_aliases(dir);
        for (k, v) in local {
            merged.insert(k, v);
        }
        merged
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_theme_config() {
        let mut cfg = Config::default();
        let custom = CustomTheme {
            name: "ocean".to_string(),
            style: "rounded".to_string(),
            line_layout: "double_line".to_string(),
            path_fg: 15,
            path_bg: 31,
            git_fg: 16,
            git_bg: 84,
            badge_fg: 16,
            badge_bg: 117,
            user_fg: None,
            user_bg: None,
            host_fg: None,
            host_bg: None,
            time_fg: Some(245),
            time_bg: Some(236),
            error_fg: Some(15),
            error_bg: Some(203),
            prompt_symbol_fg: Some(78),
            show_dev_badge: Some(true),
            use_powerline: true,
            prompt_symbol: Some("❯ ".to_string()),
        };
        cfg.custom_themes.insert("ocean".to_string(), custom);
        cfg.theme = "ocean".to_string();

        let toml_str = toml::to_string(&cfg).unwrap();
        let loaded: Config = toml::from_str(&toml_str).unwrap();

        assert_eq!(loaded.theme, "ocean");
        assert!(loaded.custom_themes.contains_key("ocean"));
        let ocean = loaded.custom_themes.get("ocean").unwrap();
        assert_eq!(ocean.path_bg, 31);
        assert_eq!(ocean.prompt_symbol.as_deref(), Some("❯ "));
    }

    #[test]
    fn test_local_aliases() {
        let mut temp_path = std::env::temp_dir();
        temp_path.push(format!("lshell_local_alias_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_path);
        fs::create_dir_all(&temp_path).unwrap();

        let mut local_map = HashMap::new();
        local_map.insert("bld".to_string(), "cargo build".to_string());
        Config::save_local_aliases(&temp_path, &local_map);

        let loaded_local = Config::load_local_aliases(&temp_path);
        assert_eq!(loaded_local.get("bld"), Some(&"cargo build".to_string()));

        let mut cfg = Config::default();
        cfg.aliases
            .insert("bld".to_string(), "global build".to_string());
        let merged = cfg.get_merged_aliases(&temp_path);
        assert_eq!(merged.get("bld"), Some(&"cargo build".to_string()));

        let empty_map = HashMap::new();
        Config::save_local_aliases(&temp_path, &empty_map);
        assert!(!temp_path.join(".lshell_alias").exists());

        let _ = fs::remove_dir_all(&temp_path);
    }
}
