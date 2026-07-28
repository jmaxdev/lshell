use crate::config::{Config, CustomTheme};
use crate::editor::{choice_multi, choice_single, input_text};
use crate::ledit::LeditEditor;
use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn builtin_alias(args: &[String], config: &mut Config) -> i32 {
    if args.is_empty() {
        println!(
            "\n {}{}Configured Aliases:{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            ResetColor
        );
        let mut keys: Vec<_> = config.aliases.keys().collect();
        keys.sort();
        for k in keys {
            println!("   {:10} = {}", k, config.aliases.get(k).unwrap());
        }
        println!();
        return 0;
    }

    if args[0] == "--save" {
        config.save();
        println!(
            " {}{}Aliases saved to ~/.lshell{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            ResetColor
        );
        return 0;
    }

    let input = args.join(" ");
    if let Some((name, val)) = input.split_once('=') {
        let clean_name = name.trim().to_string();
        let clean_val = val.trim().trim_matches('"').trim_matches('\'').to_string();
        config.aliases.insert(clean_name.clone(), clean_val.clone());
        println!(
            " {}{}Alias added: {} = '{}'{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            clean_name,
            clean_val,
            ResetColor
        );
    } else {
        eprintln!("lshell: alias: usage: alias <name>=<command> or alias --save");
        return 1;
    }

    0
}

pub fn builtin_z(prev_dir: &mut Option<PathBuf>, args: &[String]) -> i32 {
    let db = load_z_db();
    if args.is_empty() {
        println!(
            "\n {}{}Frequent Directories (z):{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            ResetColor
        );
        let mut entries: Vec<_> = db.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1));
        for (p, score) in entries.iter().take(10) {
            println!(" {:>5}  {}", score, p);
        }
        println!();
        return 0;
    }

    let query = args.join(" ").to_lowercase();
    let mut matches: Vec<_> = db
        .iter()
        .filter(|(p, _)| p.to_lowercase().contains(&query))
        .collect();

    matches.sort_by(|a, b| b.1.cmp(a.1));

    if let Some((target_path, _)) = matches.first() {
        let p = PathBuf::from(target_path);
        super::fs_cmds::builtin_cd(prev_dir, &[p.to_string_lossy().to_string()])
    } else {
        eprintln!("lshell: z: no matching directory found for '{}'", query);
        1
    }
}

pub fn load_z_db() -> HashMap<String, u32> {
    let mut db = HashMap::new();
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".lshell_z");
        if let Ok(content) = fs::read_to_string(path) {
            for line in content.lines() {
                if let Some((score_str, p)) = line.split_once('|') {
                    if let Ok(score) = score_str.parse::<u32>() {
                        db.insert(p.to_string(), score);
                    }
                }
            }
        }
    }
    db
}

pub fn save_z_db(db: &HashMap<String, u32>) {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".lshell_z");
        let mut lines: Vec<_> = db
            .iter()
            .map(|(p, score)| format!("{}|{}", score, p))
            .collect();
        lines.sort();
        let content = lines.join("\n");
        let _ = fs::write(path, content);
    }
}

pub fn record_z_visit(path: &Path) {
    let p_str = path.to_string_lossy().to_string();
    let mut db = load_z_db();
    *db.entry(p_str).or_insert(0) += 1;
    save_z_db(&db);
}

pub fn builtin_export(args: &[String]) -> i32 {
    if args.is_empty() {
        for (key, val) in env::vars() {
            println!("export {}={}", key, val);
        }
        return 0;
    }

    for arg in args {
        if let Some((key, val)) = arg.split_once('=') {
            env::set_var(key, val);
        }
    }
    0
}

pub fn builtin_secret(args: &[String], config: &mut crate::config::Config) -> i32 {
    if args.is_empty() {
        println!("lshell: secret: usage: secret <set|get|list> [KEY] [VALUE]");
        return 1;
    }

    let action = args[0].to_lowercase();
    match action.as_str() {
        "set" => {
            if args.len() < 2 {
                eprintln!(
                    "lshell: secret set: usage: secret set KEY VALUE  or  secret set KEY=VALUE"
                );
                return 1;
            }
            let (key, val) = if let Some((k, v)) = args[1].split_once('=') {
                (k.to_string(), v.to_string())
            } else if args.len() >= 3 {
                (args[1].clone(), args[2..].join(" "))
            } else {
                eprintln!("lshell: secret set: missing value for key '{}'", args[1]);
                return 1;
            };

            let encrypted = crate::security::encrypt_val(&val);
            env::set_var(&key, &val);
            config.env.insert(key.clone(), encrypted);
            config.save();

            println!(
                " {}{}🔒 Encrypted secret '{}' saved securely to ~/.lshell!{}",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(78)),
                key,
                ResetColor
            );
            0
        }

        "get" => {
            if args.len() < 2 {
                eprintln!("lshell: secret get: usage: secret get <KEY>");
                return 1;
            }
            let key = &args[1];
            if let Some(enc_val) = config.env.get(key) {
                let plain = crate::security::decrypt_val(enc_val);
                println!(" {} = {}", key, plain);
                0
            } else if let Ok(env_val) = env::var(key) {
                println!(" {} = {}", key, env_val);
                0
            } else {
                eprintln!("lshell: secret: key '{}' not found", key);
                1
            }
        }

        "list" => {
            if config.env.is_empty() {
                println!(" (no encrypted secrets stored in ~/.lshell)");
                return 0;
            }
            println!(
                "\n {}{}Encrypted Secrets (~/.lshell):{}",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(75)),
                ResetColor
            );
            for (key, val) in &config.env {
                let status = if val.starts_with("enc:") {
                    "🔒 encrypted"
                } else {
                    "plain"
                };
                println!("   {:20} ({})", key, status);
            }
            println!();
            0
        }

        _ => {
            eprintln!(
                "lshell: secret: unknown action '{}'. Use: set, get, list",
                action
            );
            1
        }
    }
}

pub fn builtin_env(args: &[String]) -> i32 {
    let filter = args.first().map(|s| s.to_lowercase());
    for (key, val) in env::vars() {
        if let Some(ref q) = filter {
            if !key.to_lowercase().contains(q) && !val.to_lowercase().contains(q) {
                continue;
            }
        }
        println!("{}={}", key, val);
    }
    0
}

pub fn builtin_unset(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("lshell: unset: usage: unset <VAR1> [VAR2 ...]");
        return 1;
    }
    for var in args {
        env::remove_var(var);
    }
    0
}

pub fn builtin_which(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("lshell: which: usage: which <command>");
        return 1;
    }

    let cmd = &args[0];
    let builtins = [
        "cd",
        "pwd",
        "ls",
        "dir",
        "tree",
        "sys",
        "info",
        "version",
        "update",
        "updater",
        "install-wt",
        "wt-install",
        "install-vscode",
        "vscode-install",
        "install",
        "cat",
        "type",
        "edit",
        "ledit",
        "touch",
        "mkdir",
        "rm",
        "del",
        "clear",
        "cls",
        "history",
        "help",
        "exit",
        "..",
        "...",
        "....",
        "search",
        "find",
        "usage",
        "du",
        "bench",
        "time",
        "top",
        "ps",
        "alias",
        "z",
        "jump",
        "export",
        "env",
        "unset",
        "head",
        "tail",
        "cp",
        "copy",
        "mv",
        "move",
    ];
    if builtins.contains(&cmd.to_lowercase().as_str()) {
        println!(" {}: lshell Built-in Command", cmd);
        return 0;
    }

    if let Ok(path_var) = env::var("PATH") {
        for dir in env::split_paths(&path_var) {
            let candidate = dir.join(cmd);
            if candidate.exists() {
                println!(" {}", candidate.display());
                return 0;
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join(format!("{}.exe", cmd));
                if candidate_exe.exists() {
                    println!(" {}", candidate_exe.display());
                    return 0;
                }
            }
        }
    }

    eprintln!("lshell: which: {} not found in PATH", cmd);
    1
}

pub fn builtin_history(history: &mut Vec<String>, args: &[String]) -> i32 {
    if !args.is_empty() {
        let arg = args[0].to_lowercase();
        if arg == "clean" || arg == "clear" || arg == "--clean" || arg == "-c" {
            history.clear();
            if let Some(home) = dirs::home_dir() {
                let path = home.join(".lshell_history");
                let _ = fs::write(&path, "");
                let _ = fs::remove_file(path);
            }
            println!(
                " {}{}✓ Command history cleared!{}",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(78)),
                ResetColor
            );
            return 0;
        }
    }

    if history.is_empty() {
        println!(" (command history is empty)");
        return 0;
    }

    println!();
    for (i, entry) in history.iter().enumerate() {
        println!(
            " {}{:4}{}  {}",
            SetForegroundColor(Color::AnsiValue(75)),
            i + 1,
            ResetColor,
            entry
        );
    }
    println!();
    0
}

pub fn builtin_help() -> i32 {
    println!();
    println!(
        " {}{}lshell - Extensible Terminal in Rust{}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(141)),
        ResetColor
    );
    println!(" Built-in Commands:");
    println!(
        "   {}ls / dir{}       List files with vector icons, colors, and sizes",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}tree [dir] [--full] [--file]{} Draw directory tree (--full for deep tree, --file to save to tree.txt)",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}sys / info{}     Display stylized system information card",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}updater / update{} Downloads and installs latest version",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}install-wt{}       Register lshell in Windows Terminal profiles",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}install-vscode{}   Register lshell as default terminal in VS Code",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}install{}          Register lshell in both Windows Terminal and VS Code",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}edit <file>{}    Real-time native text editor",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}cd / .. / ...{}  Fast directory navigation",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}pwd{}            Print current working directory",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}cat / type{}     Print numbered file contents",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}touch <file>{}   Create an empty file",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}mkdir <dir>{}    Create directories",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}rm / del <file>{} Delete files or directories",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}which <cmd>{}    Locate binary executable",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}clear / cls{}    Clear screen and scrollback history",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}history [clean]{} Display or clean command history (Search with Ctrl+R)",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}export VAR=VAL{} Set session environment variable (in-memory, temporary)",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}secret <set|get|list>{} Persistent machine-encrypted secret environment variable",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}env [query]{}    Display or filter active environment variables",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}unset <VAR>{}    Remove environment variables",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}head [-n N] <file>{} Print first N lines of file",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}tail [-n N] <file>{} Print last N lines of file",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}cp [-r] <src> <dst>{} Copy files or directories",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}mv <src> <dst>{}  Move or rename files or directories",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}z [query]{}      Smart jump to frequent directory",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}alias [k=v]{}    List/create command aliases (--save to persist)",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}search <query>{} Recursive file & content search",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}usage [dir]{}    Disk usage bar-chart visualizer",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}bench <cmd>{}    High-precision command execution benchmark",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}top / ps{}       System & process monitor card",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}theme [cmd]{}    Theme manager (list, set, create wizard, delete)",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}help{}           Display this help guide",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}exit / quit{}    Exit terminal session",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!();
    0
}

pub fn builtin_edit(args: &[String]) -> i32 {
    let filename = if args.is_empty() {
        "new_file.txt"
    } else {
        &args[0]
    };

    match LeditEditor::open(filename) {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("lshell: edit: error: {}", e);
            1
        }
    }
}

pub fn builtin_theme(args: &[String], config: &mut Config) -> i32 {
    let builtin_themes = vec![
        "minimal",
        "nord",
        "dracula",
        "catppuccin",
        "tokyonight",
        "agnoster",
    ];

    if args.is_empty() {
        let menu_options = vec![
            "Select active theme".to_string(),
            "Create custom theme (Wizard)".to_string(),
            "List all themes".to_string(),
            "Delete a custom theme".to_string(),
            "Exit".to_string(),
        ];
        match choice_single("LSHELL Theme Manager", &menu_options, 0) {
            Ok(0) => {
                let mut all_themes: Vec<String> =
                    builtin_themes.iter().map(|s| s.to_string()).collect();
                for k in config.custom_themes.keys() {
                    all_themes.push(k.clone());
                }
                let current_idx = all_themes
                    .iter()
                    .position(|t| t == &config.theme)
                    .unwrap_or(0);
                if let Ok(idx) = choice_single("Select Theme", &all_themes, current_idx) {
                    config.theme = all_themes[idx].clone();
                    config.save();
                    println!(
                        " {}{}Theme updated to: {}{}",
                        SetAttribute(Attribute::Bold),
                        SetForegroundColor(Color::AnsiValue(78)),
                        config.theme,
                        ResetColor
                    );
                }
                return 0;
            }
            Ok(1) => return theme_wizard(config),
            Ok(2) => return theme_list(builtin_themes, config),
            Ok(3) => return theme_delete_interactive(config),
            _ => return 0,
        }
    }

    match args[0].to_lowercase().as_str() {
        "list" | "ls" => theme_list(builtin_themes, config),
        "set" | "use" => {
            if args.len() < 2 {
                eprintln!("lshell: theme set: usage: theme set <theme_name>");
                return 1;
            }
            let name = &args[1];
            if builtin_themes.contains(&name.as_str()) || config.custom_themes.contains_key(name) {
                config.theme = name.to_string();
                config.save();
                println!(
                    " {}{}Theme updated to: {}{}",
                    SetAttribute(Attribute::Bold),
                    SetForegroundColor(Color::AnsiValue(78)),
                    config.theme,
                    ResetColor
                );
                0
            } else {
                eprintln!("lshell: theme set: unknown theme '{}'. Use 'theme list' to see available themes.", name);
                1
            }
        }
        "create" | "wizard" | "new" => theme_wizard(config),
        "delete" | "rm" | "del" => {
            if args.len() < 2 {
                eprintln!("lshell: theme delete: usage: theme delete <theme_name>");
                return 1;
            }
            let name = &args[1];
            if config.custom_themes.remove(name).is_some() {
                if config.theme == *name {
                    config.theme = "minimal".to_string();
                }
                config.save();
                println!(
                    " {}{}Custom theme '{}' deleted successfully.{}",
                    SetAttribute(Attribute::Bold),
                    SetForegroundColor(Color::AnsiValue(78)),
                    name,
                    ResetColor
                );
                0
            } else {
                eprintln!("lshell: theme delete: custom theme '{}' not found.", name);
                1
            }
        }
        _ => {
            eprintln!("lshell: theme: unknown command. Usage: theme [list | set <name> | create | delete <name>]");
            1
        }
    }
}

fn theme_list(builtin_themes: Vec<&str>, config: &Config) -> i32 {
    println!(
        "\n {}{}Available Themes:{}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(75)),
        ResetColor
    );
    println!(
        "   {}Built-in themes:{}",
        SetAttribute(Attribute::Bold),
        ResetColor
    );
    for t in &builtin_themes {
        if *t == config.theme {
            println!(
                "     * {}{}{} (active)",
                SetForegroundColor(Color::AnsiValue(78)),
                t,
                ResetColor
            );
        } else {
            println!("       {}", t);
        }
    }

    if !config.custom_themes.is_empty() {
        println!(
            "\n   {}Custom themes:{}",
            SetAttribute(Attribute::Bold),
            ResetColor
        );
        let mut keys: Vec<_> = config.custom_themes.keys().collect();
        keys.sort();
        for k in keys {
            if *k == config.theme {
                println!(
                    "     * {}{}{} (active)",
                    SetForegroundColor(Color::AnsiValue(78)),
                    k,
                    ResetColor
                );
            } else {
                println!("       {}", k);
            }
        }
    }
    println!();
    0
}

fn theme_wizard(config: &mut Config) -> i32 {
    println!(
        "\n {}{}=== Custom Theme Wizard ==={}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(220)),
        ResetColor
    );

    let name = match input_text("Enter name for your custom theme", Some("mytheme")) {
        Ok(val) => val.trim().to_string(),
        Err(_) => return 1,
    };

    if name.is_empty() {
        eprintln!("lshell: theme wizard: invalid theme name.");
        return 1;
    }

    let styles = vec![
        " Powerline (Solid block arrows)".to_string(),
        " Rounded / Capsule (Pill blocks)".to_string(),
        " Slanted / Diagonal (Angled blocks)".to_string(),
        "🎨 Minimal / Inline (No background fill)".to_string(),
        "📦 Brackets ([ Folder ] [ Branch ])".to_string(),
        "⚡ Pure / Modern (Clean text prompt)".to_string(),
    ];
    let style_idx = choice_single("Select Theme Design / Style", &styles, 0).unwrap_or(0);
    let chosen_style = match style_idx {
        1 => "rounded",
        2 => "slanted",
        3 => "minimal",
        4 => "brackets",
        5 => "pure",
        _ => "powerline",
    };

    let line_layouts = vec![
        "↵ Two lines (Segments on top, prompt symbol on line 2)".to_string(),
        "➔ Single line (All on one line)".to_string(),
    ];
    let layout_idx = choice_single("Select Line Layout", &line_layouts, 0).unwrap_or(0);
    let chosen_layout = if layout_idx == 1 {
        "single_line"
    } else {
        "double_line"
    };

    let color_palette = vec![
        ("Dark Gray (236)".to_string(), 236),
        ("Blue (31)".to_string(), 31),
        ("Cyan (117)".to_string(), 117),
        ("Green (84)".to_string(), 84),
        ("Purple (141)".to_string(), 141),
        ("Magenta (213)".to_string(), 213),
        ("Yellow (220)".to_string(), 220),
        ("Orange (208)".to_string(), 208),
        ("Red (203)".to_string(), 203),
        ("Light Gray (245)".to_string(), 245),
        ("White (15)".to_string(), 15),
        ("Black (16)".to_string(), 16),
    ];
    let color_names: Vec<String> = color_palette.iter().map(|(n, _)| n.clone()).collect();

    let path_bg_idx = choice_single("Select Path Background Color", &color_names, 1).unwrap_or(1);
    let path_fg_idx = choice_single("Select Path Text Color", &color_names, 10).unwrap_or(10);

    let git_bg_idx = choice_single("Select Git Background Color", &color_names, 3).unwrap_or(3);
    let git_fg_idx = choice_single("Select Git Text Color", &color_names, 11).unwrap_or(11);

    let badge_bg_idx = choice_single("Select Badge Background Color", &color_names, 6).unwrap_or(6);
    let badge_fg_idx = choice_single("Select Badge Text Color", &color_names, 11).unwrap_or(11);

    let opts = vec![
        "Enable Username Segment".to_string(),
        "Use Powerline Arrow Symbols ()".to_string(),
    ];
    let selected_opts = choice_multi("Theme Features", &opts, &[1]).unwrap_or_else(|_| vec![1]);

    let enable_user = selected_opts.contains(&0);
    let use_powerline = selected_opts.contains(&1);

    let (user_fg, user_bg) = if enable_user {
        (Some(15), Some(236))
    } else {
        (None, None)
    };

    let prompt_sym = input_text("Enter prompt symbol", Some(&config.prompt_symbol)).ok();

    let custom = CustomTheme {
        name: name.clone(),
        style: chosen_style.to_string(),
        line_layout: chosen_layout.to_string(),
        path_fg: color_palette[path_fg_idx].1,
        path_bg: color_palette[path_bg_idx].1,
        git_fg: color_palette[git_fg_idx].1,
        git_bg: color_palette[git_bg_idx].1,
        badge_fg: color_palette[badge_fg_idx].1,
        badge_bg: color_palette[badge_bg_idx].1,
        user_fg,
        user_bg,
        use_powerline,
        prompt_symbol: prompt_sym,
    };

    config.custom_themes.insert(name.clone(), custom);
    config.theme = name.clone();
    config.save();

    println!(
        "\n {}{}✨ Custom theme '{}' created and activated!{}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(78)),
        name,
        ResetColor
    );
    0
}

fn theme_delete_interactive(config: &mut Config) -> i32 {
    if config.custom_themes.is_empty() {
        println!(
            " {}{}No custom themes found to delete.{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(208)),
            ResetColor
        );
        return 0;
    }

    let custom_keys: Vec<String> = config.custom_themes.keys().cloned().collect();
    if let Ok(idx) = choice_single("Select Custom Theme to Delete", &custom_keys, 0) {
        let name = &custom_keys[idx];
        config.custom_themes.remove(name);
        if config.theme == *name {
            config.theme = "minimal".to_string();
        }
        config.save();
        println!(
            " {}{}Custom theme '{}' deleted. Active theme set to 'minimal'.{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            name,
            ResetColor
        );
    }
    0
}
