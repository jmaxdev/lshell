use crate::config::Config;
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
        let mut lines: Vec<_> = db.iter().map(|(p, score)| format!("{}|{}", score, p)).collect();
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
        "cd", "pwd", "ls", "dir", "tree", "sys", "info", "version", "update", "updater",
        "install-wt", "wt-install", "install-vscode", "vscode-install", "install", "cat",
        "type", "edit", "ledit", "touch", "mkdir", "rm", "del", "clear", "cls", "history",
        "help", "exit", "..", "...", "....", "search", "find", "usage", "du", "bench", "time",
        "top", "ps", "alias", "z", "jump", "export", "env", "unset", "head", "tail", "cp", "copy",
        "mv", "move"
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
        "   {}history{}        Display command history (Search with Ctrl+R)",
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
        "   {}env [query]{}    Display or filter environment variables",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}unset <VAR>{}    Remove environment variables",
        SetForegroundColor(Color::AnsiValue(78)),
        ResetColor
    );
    println!(
        "   {}export VAR=VAL{} Define environment variables",
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
