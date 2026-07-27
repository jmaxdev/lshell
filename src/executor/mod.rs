use crate::config::Config;
use crate::ledit::LeditEditor;
use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{stdout, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Executor {
    history: Vec<String>,
    prev_dir: Option<PathBuf>,
}

impl Executor {
    pub fn new() -> Self {
        let history = Self::load_history();
        Self {
            history,
            prev_dir: None,
        }
    }

    pub fn execute(&mut self, input: &str, config: &mut Config) -> i32 {
        let expanded_env = expand_env_vars(input);
        let trimmed = expanded_env.trim();
        if trimmed.is_empty() {
            return 0;
        }

        if trimmed == ".." {
            self.add_history(trimmed);
            return self.builtin_cd(&["..".to_string()]);
        } else if trimmed == "..." {
            self.add_history(trimmed);
            return self.builtin_cd(&["../..".to_string()]);
        } else if trimmed == "...." {
            self.add_history(trimmed);
            return self.builtin_cd(&["../../..".to_string()]);
        }

        let expanded = self.expand_alias(trimmed, config);
        let parts = parse_command_line(&expanded);

        if parts.is_empty() {
            return 0;
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        let code = match cmd.as_str() {
            "exit" | "quit" => std::process::exit(0),
            "cd" => self.builtin_cd(args),
            "z" | "jump" => self.builtin_z(args),
            "alias" => self.builtin_alias(args, config),
            "search" | "find" => self.builtin_search(args),
            "usage" | "du" => self.builtin_usage(args),
            "bench" | "time" => self.builtin_bench(args, config),
            "top" | "ps" => self.builtin_top(),
            "pwd" => self.builtin_pwd(),
            "clear" | "cls" => self.builtin_clear(),
            "ls" | "dir" => self.builtin_ls(args),
            "tree" => self.builtin_tree(args),
            "sys" | "info" => self.builtin_sys(),
            "version" | "--version" | "-v" => self.builtin_version(),
            "update" | "updater" => self.builtin_update(),
            "install-wt" | "wt-install" => self.builtin_install_wt(),
            "install-vscode" | "vscode-install" => self.builtin_install_vscode(),
            "install" => self.builtin_install(args),
            "cat" | "type" => self.builtin_cat(args),
            "edit" | "ledit" => self.builtin_edit(args),
            "touch" => self.builtin_touch(args),
            "mkdir" => self.builtin_mkdir(args),
            "rm" | "del" => self.builtin_rm(args),
            "which" | "where" => self.builtin_which(args),
            "history" => self.builtin_history(),
            "help" => self.builtin_help(),
            "export" => self.builtin_export(args),
            _ => self.run_external(&parts[0], args),
        };

        if code == 0 {
            self.add_history(trimmed);
        }

        code
    }

    fn expand_alias(&self, input: &str, config: &Config) -> String {
        let mut parts = input.split_whitespace();
        if let Some(first) = parts.next() {
            if let Some(alias_val) = config.aliases.get(first) {
                let remainder: Vec<&str> = parts.collect();
                if remainder.is_empty() {
                    return alias_val.clone();
                } else {
                    return format!("{} {}", alias_val, remainder.join(" "));
                }
            }
        }
        input.to_string()
    }

    fn builtin_cd(&mut self, args: &[String]) -> i32 {
        let target = if args.is_empty() || args[0] == "~" {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        } else if args[0] == "-" {
            if let Some(ref prev) = self.prev_dir {
                prev.clone()
            } else {
                eprintln!("lshell: OLDPWD not set");
                return 1;
            }
        } else {
            PathBuf::from(&args[0])
        };

        let current = env::current_dir().ok();
        match env::set_current_dir(&target) {
            Ok(_) => {
                self.prev_dir = current;
                if let Ok(cwd) = env::current_dir() {
                    record_z_visit(&cwd);
                }
                0
            }
            Err(e) => {
                eprintln!("lshell: cd: {}: {}", target.display(), e);
                1
            }
        }
    }

    fn builtin_z(&mut self, args: &[String]) -> i32 {
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
            self.builtin_cd(&[p.to_string_lossy().to_string()])
        } else {
            eprintln!("lshell: z: no matching directory found for '{}'", query);
            1
        }
    }

    fn builtin_alias(&self, args: &[String], config: &mut Config) -> i32 {
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

    fn builtin_search(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: search: usage: search <query> [path]");
            return 1;
        }

        let query = &args[0];
        let target_dir = if args.len() >= 2 {
            PathBuf::from(&args[1])
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        println!(
            "\n {}{}Searching for '{}' in {}...{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            query,
            target_dir.display(),
            ResetColor
        );

        let mut count = 0;
        search_recursive(&target_dir, query, &mut count, 50);
        println!(
            " {}{}Found {} matches.{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(245)),
            count,
            ResetColor
        );
        0
    }

    fn builtin_usage(&self, args: &[String]) -> i32 {
        let target = if !args.is_empty() {
            PathBuf::from(&args[0])
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        println!(
            "\n {}{}Disk Usage Analysis: {}{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            target.display(),
            ResetColor
        );

        let mut items = Vec::new();
        let mut total_size = 0u64;

        if let Ok(entries) = fs::read_dir(&target) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') {
                    continue;
                }
                let path = entry.path();
                let size = get_dir_size(&path);
                total_size += size;
                items.push((name, size, path.is_dir()));
            }
        }

        items.sort_by(|a, b| b.1.cmp(&a.1));

        for (name, size, is_dir) in items {
            let pct = if total_size > 0 {
                (size as f64 / total_size as f64 * 100.0) as usize
            } else {
                0
            };
            let bar_len = pct / 10;
            let bar = format!("[{}{}]", "█".repeat(bar_len), "░".repeat(10 - bar_len));
            let icon = if is_dir { "" } else { "📄" };

            println!(
                "   {} {:>4}% {} {:>10}  {} {}",
                SetForegroundColor(Color::AnsiValue(78)),
                pct,
                bar,
                format_bytes(size),
                icon,
                name
            );
            print!("{}", ResetColor);
        }
        println!("\n   Total: {}\n", format_bytes(total_size));
        0
    }

    fn builtin_bench(&mut self, args: &[String], config: &mut Config) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: bench: usage: bench <command...>");
            return 1;
        }

        let cmd_str = args.join(" ");
        println!(
            "\n {}{}⏱️  Benchmarking: {}{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            cmd_str,
            ResetColor
        );

        let start = std::time::Instant::now();
        let code = self.execute(&cmd_str, config);
        let elapsed = start.elapsed();

        println!(
            "\n {}{}⚡ Command finished in {:.2?} (exit code {}){}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            elapsed,
            code,
            ResetColor
        );

        code
    }

    fn builtin_top(&self) -> i32 {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        let pid = std::process::id();
        let cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

        println!();
        println!(
            " {}{}+------------------------------------------------------+{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!(
            " {}{}|               SYSTEM & PROCESS MONITOR               |{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!(
            " {}{}+------------------------------------------------------+{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!("   {}Operating System:{}   {}", SetForegroundColor(Color::AnsiValue(75)), ResetColor, os);
        println!("   {}Architecture:    {}   {}", SetForegroundColor(Color::AnsiValue(75)), ResetColor, arch);
        println!("   {}CPU Cores:       {}   {}", SetForegroundColor(Color::AnsiValue(75)), ResetColor, cpus);
        println!("   {}Current PID:     {}   {}", SetForegroundColor(Color::AnsiValue(75)), ResetColor, pid);
        println!("   {}Memory Subsystem:{}   64-bit Virtual Memory", SetForegroundColor(Color::AnsiValue(75)), ResetColor);
        println!(
            " {}{}+------------------------------------------------------+{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );

        0
    }

    fn builtin_pwd(&self) -> i32 {
        if let Ok(dir) = env::current_dir() {
            println!(
                " {}{}{}",
                SetForegroundColor(Color::AnsiValue(75)),
                dir.display(),
                ResetColor
            );
            0
        } else {
            1
        }
    }

    fn builtin_tree(&self, args: &[String]) -> i32 {
        let mut full = false;
        let mut to_file = false;
        let mut target_path = None;

        for arg in args {
            if arg == "--full" {
                full = true;
            } else if arg == "--file" {
                to_file = true;
            } else if !arg.starts_with('-') && target_path.is_none() {
                target_path = Some(PathBuf::from(arg));
            }
        }

        let target = target_path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
        let max_depth = if full { 3 } else { 1 };

        println!(
            "\n {}{} {}{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            target.display(),
            ResetColor
        );

        let mut file_buf = if to_file {
            Some(format!(" {}\n", target.display()))
        } else {
            None
        };

        render_tree_recursive(&target, "", 0, max_depth, &mut file_buf);
        println!();

        if let Some(content) = file_buf {
            let out_file = PathBuf::from("tree.txt");
            match fs::write(&out_file, content) {
                Ok(_) => println!(
                    " {}{}Tree structure saved to tree.txt{}",
                    SetAttribute(Attribute::Bold),
                    SetForegroundColor(Color::AnsiValue(78)),
                    ResetColor
                ),
                Err(e) => eprintln!("lshell: tree: failed to write tree.txt: {}", e),
            }
        }

        0
    }

    fn builtin_sys(&self) -> i32 {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        let username = env::var("USERNAME").or_else(|_| env::var("USER")).unwrap_or_else(|_| "lshell".to_string());
        let rust_ver = "1.85.0";
        let app_ver = env!("CARGO_PKG_VERSION");

        println!();
        println!(
            " {}{}+------------------------------------------------------+{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!(
            " {}|  ⚡ lshell System Information                              |{}",
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!(
            " {}{}+------------------------------------------------------+{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );
        println!(
            "   📦 Shell Version:  {}{}{}",
            SetForegroundColor(Color::AnsiValue(150)),
            app_ver,
            ResetColor
        );
        println!(
            "   👤 User:          {}{}{}",
            SetForegroundColor(Color::AnsiValue(78)),
            username,
            ResetColor
        );
        println!(
            "   💻 OS:            {}{}{}",
            SetForegroundColor(Color::AnsiValue(75)),
            os,
            ResetColor
        );
        println!(
            "   ⚙  Architecture:  {}{}{}",
            SetForegroundColor(Color::AnsiValue(220)),
            arch,
            ResetColor
        );
        println!(
            "   🦀 Rust Core:     {}{}{}",
            SetForegroundColor(Color::AnsiValue(208)),
            rust_ver,
            ResetColor
        );
        println!(
            " {}{}+------------------------------------------------------+{}\n",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(141)),
            ResetColor
        );

        0
    }

    pub fn builtin_version(&self) -> i32 {
        println!(
            " {}{}lshell v{}{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            env!("CARGO_PKG_VERSION"),
            ResetColor
        );
        0
    }

    pub fn check_update_banner(&self) {
        if let Ok(builder) = self_update::backends::github::Update::configure()
            .repo_owner("jmaxdev")
            .repo_name("lshell")
            .bin_name("lshell")
            .current_version(self_update::cargo_crate_version!())
            .build()
        {
            if let Ok(latest) = builder.get_latest_release() {
                let latest_ver = latest.version.trim_start_matches('v');
                let current_ver = self_update::cargo_crate_version!();
                if latest_ver != current_ver {
                    println!(
                        "\n {}{}+-----------------------------------------------------------+{}",
                        SetAttribute(Attribute::Bold),
                        SetForegroundColor(Color::AnsiValue(214)),
                        ResetColor
                    );
                    println!(
                        " {}|  Notice: A new version of lshell is available ({:<10}) |{}",
                        SetForegroundColor(Color::AnsiValue(214)),
                        latest.version,
                        ResetColor
                    );
                    println!(
                        " {}|  Run 'lshell updater' or 'updater' to update.            |{}",
                        SetForegroundColor(Color::AnsiValue(214)),
                        ResetColor
                    );
                    println!(
                        " {}{}+-----------------------------------------------------------+{}\n",
                        SetAttribute(Attribute::Bold),
                        SetForegroundColor(Color::AnsiValue(214)),
                        ResetColor
                    );
                }
            }
        }
    }

    pub fn builtin_update(&self) -> i32 {
        println!();
        println!(
            " {}{}Checking for updates...{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            ResetColor
        );

        let updater_status = self_update::backends::github::Update::configure()
            .repo_owner("jmaxdev")
            .repo_name("lshell")
            .bin_name("lshell")
            .show_download_progress(true)
            .current_version(self_update::cargo_crate_version!())
            .build();

        match updater_status {
            Ok(builder) => match builder.update() {
                Ok(rel) => {
                    if rel.updated() {
                        println!(
                            "\n {}{}Successfully updated lshell to version {}! Exiting...{}",
                            SetAttribute(Attribute::Bold),
                            SetForegroundColor(Color::AnsiValue(78)),
                            rel.version(),
                            ResetColor
                        );
                        std::process::exit(0);
                    } else {
                        println!(" lshell is already up to date (version {}).", rel.version());
                        0
                    }
                }
                Err(e) => {
                    eprintln!("lshell: update: failed to update: {}", e);
                    1
                }
            },
            Err(e) => {
                eprintln!("lshell: update: failed to configure updater: {}", e);
                1
            }
        }
    }

    pub fn builtin_install_wt(&self) -> i32 {
        println!();
        println!(
            " {}{}Installing lshell profile into Windows Terminal...{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            ResetColor
        );

        let exe_path = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("lshell: install-wt: failed to determine executable path: {}", e);
                return 1;
            }
        };

        let local_app_data = match env::var("LOCALAPPDATA") {
            Ok(val) => PathBuf::from(val),
            Err(_) => {
                eprintln!("lshell: install-wt: LOCALAPPDATA environment variable not found");
                return 1;
            }
        };

        let candidate_paths = [
            local_app_data.join("Packages\\Microsoft.WindowsTerminal_8wekyb3d8bbwe\\LocalState\\settings.json"),
            local_app_data.join("Packages\\Microsoft.WindowsTerminalPreview_8wekyb3d8bbwe\\LocalState\\settings.json"),
            local_app_data.join("Microsoft\\Windows Terminal\\settings.json"),
        ];

        let mut settings_path = None;
        for path in &candidate_paths {
            if path.exists() {
                settings_path = Some(path.clone());
                break;
            }
        }

        let settings_path = match settings_path {
            Some(p) => p,
            None => {
                eprintln!("lshell: install-wt: Windows Terminal settings.json not found in standard locations");
                return 1;
            }
        };

        let content = match fs::read_to_string(&settings_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("lshell: install-wt: failed to read {}: {}", settings_path.display(), e);
                return 1;
            }
        };

        let mut val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("lshell: install-wt: failed to parse JSON in {}: {}", settings_path.display(), e);
                return 1;
            }
        };

        let guid = "{a6c8e547-817d-4c3e-96a8-f3d99e098711}";
        let exe_str = exe_path.to_string_lossy().to_string();

        let new_profile = serde_json::json!({
            "guid": guid,
            "name": "lshell",
            "commandline": exe_str,
            "startingDirectory": "%USERPROFILE%",
            "icon": exe_str
        });

        if let Some(profiles) = val.get_mut("profiles") {
            if let Some(list) = profiles.get_mut("list").and_then(|l| l.as_array_mut()) {
                let mut exists = false;
                for p in list.iter_mut() {
                    if p.get("guid").and_then(|g| g.as_str()) == Some(guid)
                        || p.get("name").and_then(|n| n.as_str()) == Some("lshell")
                    {
                        *p = new_profile.clone();
                        exists = true;
                        break;
                    }
                }
                if !exists {
                    list.push(new_profile);
                }
            } else {
                eprintln!("lshell: install-wt: invalid profiles.list format in settings.json");
                return 1;
            }
        } else {
            eprintln!("lshell: install-wt: invalid profiles object in settings.json");
            return 1;
        }

        let updated_content = match serde_json::to_string_pretty(&val) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("lshell: install-wt: failed to serialize updated JSON: {}", e);
                return 1;
            }
        };

        if let Err(e) = fs::write(&settings_path, updated_content) {
            eprintln!("lshell: install-wt: failed to write updated settings.json: {}", e);
            return 1;
        }

        println!(
            "\n {}{}Successfully registered lshell profile in Windows Terminal!{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(78)),
            ResetColor
        );
        println!(" Settings updated at: {}\n", settings_path.display());
        0
    }

    pub fn builtin_install_vscode(&self) -> i32 {
        println!();
        println!(
            " {}{}Installing lshell profile into VS Code...{}",
            SetAttribute(Attribute::Bold),
            SetForegroundColor(Color::AnsiValue(75)),
            ResetColor
        );

        let exe_path = match env::current_exe() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("lshell: install-vscode: failed to determine executable path: {}", e);
                return 1;
            }
        };
        let exe_str = exe_path.to_string_lossy().to_string();

        let mut target_paths = Vec::new();

        if cfg!(target_os = "windows") {
            if let Ok(appdata) = env::var("APPDATA") {
                target_paths.push(PathBuf::from(appdata).join("Code\\User\\settings.json"));
            }
        } else if cfg!(target_os = "macos") {
            if let Some(home) = dirs::home_dir() {
                target_paths.push(home.join("Library/Application Support/Code/User/settings.json"));
            }
        } else {
            if let Some(home) = dirs::home_dir() {
                target_paths.push(home.join(".config/Code/User/settings.json"));
            }
        }

        if let Ok(cwd) = env::current_dir() {
            let local_vscode = cwd.join(".vscode");
            if local_vscode.exists() {
                target_paths.push(local_vscode.join("settings.json"));
            }
        }

        let mut count = 0;
        for path in target_paths {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            let mut val: serde_json::Value = if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
                } else {
                    serde_json::json!({})
                }
            } else {
                serde_json::json!({})
            };

            let os_key = if cfg!(target_os = "windows") {
                "windows"
            } else if cfg!(target_os = "macos") {
                "osx"
            } else {
                "linux"
            };

            let profiles_key = format!("terminal.integrated.profiles.{}", os_key);
            let default_key = format!("terminal.integrated.defaultProfile.{}", os_key);

            let profile_entry = serde_json::json!({
                "path": exe_str,
                "icon": "terminal-bash"
            });

            if let Some(obj) = val.as_object_mut() {
                let profiles = obj.entry(&profiles_key).or_insert_with(|| serde_json::json!({}));
                if let Some(p_map) = profiles.as_object_mut() {
                    p_map.insert("lshell".to_string(), profile_entry);
                }
                obj.insert(default_key, serde_json::Value::String("lshell".to_string()));
            }

            if let Ok(content) = serde_json::to_string_pretty(&val) {
                if fs::write(&path, content).is_ok() {
                    println!(" Updated VS Code settings at: {}", path.display());
                    count += 1;
                }
            }
        }

        if count > 0 {
            println!(
                "\n {}{}Successfully registered lshell in VS Code!{}",
                SetAttribute(Attribute::Bold),
                SetForegroundColor(Color::AnsiValue(78)),
                ResetColor
            );
            0
        } else {
            eprintln!("lshell: install-vscode: could not write VS Code settings.json");
            1
        }
    }

    pub fn builtin_install(&self, args: &[String]) -> i32 {
        let sub = args.get(1).map(|s| s.to_lowercase());
        match sub.as_deref() {
            Some("wt") | Some("windows-terminal") => self.builtin_install_wt(),
            Some("vscode") | Some("code") => self.builtin_install_vscode(),
            _ => {
                let code_wt = self.builtin_install_wt();
                let code_vc = self.builtin_install_vscode();
                if code_wt == 0 || code_vc == 0 { 0 } else { 1 }
            }
        }
    }

    fn builtin_ls(&self, args: &[String]) -> i32 {
        let target_dir = if !args.is_empty() && !args[0].starts_with('-') {
            PathBuf::from(&args[0])
        } else {
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        let show_hidden = args.iter().any(|a| a.contains('a'));

        let entries = match fs::read_dir(&target_dir) {
            Ok(read) => read,
            Err(e) => {
                eprintln!("lshell: ls: {}: {}", target_dir.display(), e);
                return 1;
            }
        };

        let mut items = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            let metadata = entry.metadata().ok();
            let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            items.push((name, is_dir, size));
        }

        items.sort_by(|a, b| {
            if a.1 != b.1 {
                b.1.cmp(&a.1)
            } else {
                a.0.to_lowercase().cmp(&b.0.to_lowercase())
            }
        });

        println!();
        for (name, is_dir, size) in items {
            let (icon, name_color, bold) = get_file_style(&name, is_dir);

            let formatted_size = if is_dir {
                format!("{:<10}", "<DIR>")
            } else {
                format!("{:<10}", format_bytes(size))
            };

            let dir_suffix = if is_dir { "/" } else { "" };
            let display_name = format!("{}{}", name, dir_suffix);

            if bold {
                print!(
                    " {:<2} {}{}{:<26}{} {}{}{}\n",
                    icon,
                    SetAttribute(Attribute::Bold),
                    SetForegroundColor(name_color),
                    display_name,
                    ResetColor,
                    SetForegroundColor(Color::AnsiValue(245)),
                    formatted_size,
                    ResetColor
                );
            } else {
                print!(
                    " {:<2} {}{:<26}{} {}{}{}\n",
                    icon,
                    SetForegroundColor(name_color),
                    display_name,
                    ResetColor,
                    SetForegroundColor(Color::AnsiValue(245)),
                    formatted_size,
                    ResetColor
                );
            }
        }
        println!();

        0
    }

    fn builtin_cat(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: cat: usage: cat <file>");
            return 1;
        }

        for path_str in args {
            let path = PathBuf::from(path_str);
            match fs::read_to_string(&path) {
                Ok(content) => {
                    println!();
                    for (line_idx, line) in content.lines().enumerate() {
                        println!(
                            " {}{:4}{} | {}",
                            SetForegroundColor(Color::AnsiValue(242)),
                            line_idx + 1,
                            ResetColor,
                            line
                        );
                    }
                    println!();
                }
                Err(e) => {
                    eprintln!("lshell: cat: {}: {}", path.display(), e);
                    return 1;
                }
            }
        }
        0
    }

    fn builtin_edit(&self, args: &[String]) -> i32 {
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

    fn builtin_touch(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: touch: usage: touch <file>");
            return 1;
        }

        for path_str in args {
            let path = PathBuf::from(path_str);
            if !path.exists() {
                if let Err(e) = fs::File::create(&path) {
                    eprintln!("lshell: touch: {}: {}", path.display(), e);
                    return 1;
                }
            }
        }
        0
    }

    fn builtin_mkdir(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: mkdir: usage: mkdir <directory>");
            return 1;
        }

        for path_str in args {
            if let Err(e) = fs::create_dir_all(path_str) {
                eprintln!("lshell: mkdir: {}: {}", path_str, e);
                return 1;
            }
        }
        0
    }

    fn builtin_rm(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: rm: usage: rm <file_or_directory>");
            return 1;
        }

        for path_str in args {
            let path = PathBuf::from(path_str);
            if path.is_dir() {
                if let Err(e) = fs::remove_dir_all(&path) {
                    eprintln!("lshell: rm: {}: {}", path.display(), e);
                    return 1;
                }
            } else if path.exists() {
                if let Err(e) = fs::remove_file(&path) {
                    eprintln!("lshell: rm: {}: {}", path.display(), e);
                    return 1;
                }
            }
        }
        0
    }

    fn builtin_which(&self, args: &[String]) -> i32 {
        if args.is_empty() {
            eprintln!("lshell: which: usage: which <command>");
            return 1;
        }

        let cmd = &args[0];
        let builtins = ["cd", "pwd", "ls", "dir", "tree", "sys", "info", "version", "update", "updater", "install-wt", "wt-install", "install-vscode", "vscode-install", "install", "cat", "type", "edit", "ledit", "touch", "mkdir", "rm", "del", "clear", "cls", "history", "help", "exit", "..", "...", "...."];
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

    fn builtin_clear(&self) -> i32 {
        let mut out = stdout();
        let _ = execute!(
            out,
            Clear(ClearType::Purge),
            Clear(ClearType::All),
            MoveTo(0, 0)
        );
        let _ = out.write_all(b"\x1b[3J\x1b[H\x1b[2J");
        let _ = out.flush();
        0
    }

    fn builtin_history(&self) -> i32 {
        println!();
        for (i, entry) in self.history.iter().enumerate() {
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

    fn builtin_help(&self) -> i32 {
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

    fn builtin_export(&self, args: &[String]) -> i32 {
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

    fn run_external(&self, cmd: &str, args: &[String]) -> i32 {
        let mut command = Command::new(cmd);
        command.args(args);

        match command.status() {
            Ok(status) => status.code().unwrap_or(0),
            Err(e) => {
                #[cfg(windows)]
                {
                    let mut cmd_shell = Command::new("cmd.exe");
                    cmd_shell.arg("/C").arg(cmd).args(args);
                    if let Ok(status) = cmd_shell.status() {
                        return status.code().unwrap_or(0);
                    }
                }

                eprintln!("lshell: command not found: {} ({})", cmd, e);
                127
            }
        }
    }

    fn add_history(&mut self, entry: &str) {
        self.history.push(entry.to_string());
        self.save_history();
    }

    fn save_history(&self) {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".lshell_history");
            let content = self.history.join("\n");
            let _ = fs::write(path, content);
        }
    }

    fn load_history() -> Vec<String> {
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".lshell_history");
            if let Ok(content) = fs::read_to_string(path) {
                return content
                    .lines()
                    .map(|s| s.to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
        }
        Vec::new()
    }

    pub fn get_history(&self) -> &[String] {
        &self.history
    }
}

fn render_tree_recursive(
    dir: &Path,
    prefix: &str,
    depth: usize,
    max_depth: usize,
    file_buf: &mut Option<String>,
) {
    if depth >= max_depth {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        let mut items: Vec<_> = entries
            .flatten()
            .filter(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .collect();

        items.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        let total = items.len();
        for (i, entry) in items.iter().enumerate() {
            let is_last = i == total - 1;
            let branch = if is_last { "└── " } else { "├── " };
            let child_prefix = if is_last { "    " } else { "│   " };

            let name = entry.file_name().to_string_lossy().to_string();
            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let (icon, name_color, _) = get_file_style(&name, is_dir);

            println!(
                " {}{}{}{}{:<2} {}{}",
                SetForegroundColor(Color::AnsiValue(242)),
                prefix,
                branch,
                ResetColor,
                icon,
                SetForegroundColor(name_color),
                name
            );
            print!("{}", ResetColor);

            if let Some(buf) = file_buf.as_mut() {
                buf.push_str(&format!(" {}{}{:<2} {}\n", prefix, branch, icon, name));
            }

            if is_dir && depth + 1 < max_depth {
                let mut next_prefix = prefix.to_string();
                next_prefix.push_str(child_prefix);
                render_tree_recursive(&entry.path(), &next_prefix, depth + 1, max_depth, file_buf);
            }
        }
    }
}

fn get_file_style(name: &str, is_dir: bool) -> (&'static str, Color, bool) {
    if is_dir {
        return ("\u{F07B}", Color::AnsiValue(75), true);
    }

    let lower = name.to_lowercase();
    if lower.ends_with(".rs") {
        ("\u{E7A8}", Color::AnsiValue(208), false)
    } else if lower.ends_with(".ts") || lower.ends_with(".tsx") {
        ("\u{E628}", Color::AnsiValue(39), false)
    } else if lower.ends_with(".js") || lower.ends_with(".jsx") || lower.ends_with(".mjs") {
        ("\u{E60C}", Color::AnsiValue(220), false)
    } else if lower.ends_with(".py") {
        ("\u{E606}", Color::AnsiValue(214), false)
    } else if lower.ends_with(".html") || lower.ends_with(".htm") {
        ("\u{E736}", Color::AnsiValue(202), false)
    } else if lower.ends_with(".css") || lower.ends_with(".scss") {
        ("\u{E749}", Color::AnsiValue(39), false)
    } else if lower.ends_with(".c") || lower.ends_with(".cpp") || lower.ends_with(".h") {
        ("\u{E61D}", Color::AnsiValue(75), false)
    } else if lower.ends_with(".go") {
        ("\u{E627}", Color::AnsiValue(80), false)
    } else if lower.ends_with(".java") || lower.ends_with(".kt") {
        ("\u{E738}", Color::AnsiValue(166), false)
    } else if lower.ends_with(".toml") || lower.ends_with(".json") || lower.ends_with(".yaml") || lower.ends_with(".yml") {
        ("\u{E615}", Color::AnsiValue(220), false)
    } else if lower.ends_with(".lock") {
        ("\u{F023}", Color::AnsiValue(245), false)
    } else if lower.starts_with('.') {
        ("\u{F418}", Color::AnsiValue(242), false)
    } else if lower.ends_with(".exe") || lower.ends_with(".bat") || lower.ends_with(".cmd") || lower.ends_with(".sh") {
        ("\u{F0E7}", Color::AnsiValue(78), true)
    } else if lower.ends_with(".md") || lower.ends_with(".txt") {
        ("\u{E609}", Color::AnsiValue(111), false)
    } else if lower.ends_with(".png") || lower.ends_with(".jpg") || lower.ends_with(".jpeg") || lower.ends_with(".svg") || lower.ends_with(".ico") {
        ("\u{F03E}", Color::AnsiValue(177), false)
    } else if lower.ends_with(".zip") || lower.ends_with(".tar") || lower.ends_with(".gz") || lower.ends_with(".7z") {
        ("\u{F410}", Color::AnsiValue(172), false)
    } else {
        ("\u{F15B}", Color::AnsiValue(252), false)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn load_z_db() -> HashMap<String, u32> {
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

fn save_z_db(db: &HashMap<String, u32>) {
    if let Some(home) = dirs::home_dir() {
        let path = home.join(".lshell_z");
        let mut lines: Vec<_> = db.iter().map(|(p, score)| format!("{}|{}", score, p)).collect();
        lines.sort();
        let content = lines.join("\n");
        let _ = fs::write(path, content);
    }
}

fn record_z_visit(path: &Path) {
    let p_str = path.to_string_lossy().to_string();
    let mut db = load_z_db();
    *db.entry(p_str).or_insert(0) += 1;
    save_z_db(&db);
}

fn expand_env_vars(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '$' {
            let mut var_name = String::new();
            while let Some(&next_ch) = chars.peek() {
                if next_ch.is_alphanumeric() || next_ch == '_' {
                    var_name.push(chars.next().unwrap());
                } else {
                    break;
                }
            }
            if !var_name.is_empty() {
                if let Ok(val) = env::var(&var_name) {
                    result.push_str(&val);
                } else {
                    result.push('$');
                    result.push_str(&var_name);
                }
            } else {
                result.push('$');
            }
        } else {
            result.push(ch);
        }
    }

    result
}

fn search_recursive(dir: &Path, query: &str, count: &mut usize, max_results: usize) {
    if *count >= max_results {
        return;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if *count >= max_results {
                break;
            }
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') || name == "target" || name == "node_modules" {
                continue;
            }

            if path.is_dir() {
                search_recursive(&path, query, count, max_results);
            } else if path.is_file() {
                if name.to_lowercase().contains(&query.to_lowercase()) {
                    println!(
                        "   {}📄 File match: {}{}",
                        SetForegroundColor(Color::AnsiValue(78)),
                        path.display(),
                        ResetColor
                    );
                    *count += 1;
                }

                if let Ok(content) = fs::read_to_string(&path) {
                    for (line_no, line) in content.lines().enumerate() {
                        if *count >= max_results {
                            break;
                        }
                        if line.to_lowercase().contains(&query.to_lowercase()) {
                            println!(
                                "   {}:{}{} {} {}",
                                SetForegroundColor(Color::AnsiValue(75)),
                                path.display(),
                                SetForegroundColor(Color::AnsiValue(245)),
                                line_no + 1,
                                ResetColor
                            );
                            println!("      {}", line.trim());
                            *count += 1;
                        }
                    }
                }
            }
        }
    }
}

fn get_dir_size(path: &Path) -> u64 {
    if path.is_file() {
        return fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            total += get_dir_size(&entry.path());
        }
    }
    total
}

fn parse_command_line(input: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = ' ';

    for ch in input.chars() {
        match ch {
            '"' | '\'' if !in_quotes => {
                in_quotes = true;
                quote_char = ch;
            }
            c if in_quotes && c == quote_char => {
                in_quotes = false;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    args.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        args.push(current);
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tree_flags() {
        let executor = Executor::new();
        let mut temp_path = env::temp_dir();
        temp_path.push(format!("lshell_tree_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&temp_path);
        fs::create_dir_all(&temp_path).unwrap();

        let sub_dir = temp_path.join("subdir");
        fs::create_dir(&sub_dir).unwrap();
        fs::write(temp_path.join("file1.txt"), "hello").unwrap();
        fs::write(sub_dir.join("file2.txt"), "world").unwrap();

        let orig_dir = env::current_dir().unwrap();
        env::set_current_dir(&temp_path).unwrap();

        let res = executor.builtin_tree(&[]);
        assert_eq!(res, 0);
        assert!(!temp_path.join("tree.txt").exists());

        let res = executor.builtin_tree(&["--file".to_string()]);
        assert_eq!(res, 0);
        let file_path = temp_path.join("tree.txt");
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("file1.txt"));
        assert!(content.contains("subdir"));
        assert!(!content.contains("file2.txt"));
        fs::remove_file(&file_path).unwrap();

        let res = executor.builtin_tree(&["--full".to_string(), "--file".to_string()]);
        assert_eq!(res, 0);
        assert!(file_path.exists());
        let full_content = fs::read_to_string(&file_path).unwrap();
        assert!(full_content.contains("file1.txt"));
        assert!(full_content.contains("subdir"));
        assert!(full_content.contains("file2.txt"));

        env::set_current_dir(orig_dir).unwrap();
        let _ = fs::remove_dir_all(&temp_path);
    }

    #[test]
    fn test_env_var_expansion() {
        env::set_var("TEST_LSHELL_VAR", "WORLD");
        let expanded = expand_env_vars("echo HELLO_$TEST_LSHELL_VAR");
        assert_eq!(expanded, "echo HELLO_WORLD");
    }

    #[test]
    fn test_alias_and_export() {
        let executor = Executor::new();
        let mut config = Config::default();

        let res = executor.builtin_alias(&["mycls=clear".to_string()], &mut config);
        assert_eq!(res, 0);
        assert_eq!(config.aliases.get("mycls"), Some(&"clear".to_string()));

        let res = executor.builtin_export(&["MY_VAR=TEST123".to_string()]);
        assert_eq!(res, 0);
        assert_eq!(env::var("MY_VAR").unwrap(), "TEST123");
    }
}
