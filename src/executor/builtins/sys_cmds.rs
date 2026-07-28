use crossterm::{
    cursor::MoveTo,
    execute,
    style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::env;
use std::fs;
use std::io::{stdout, Write};
use std::path::PathBuf;

pub fn builtin_top() -> i32 {
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

pub fn builtin_sys() -> i32 {
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

pub fn builtin_version() -> i32 {
    println!(
        " {}{}lshell v{}{}",
        SetAttribute(Attribute::Bold),
        SetForegroundColor(Color::AnsiValue(78)),
        env!("CARGO_PKG_VERSION"),
        ResetColor
    );
    0
}

pub fn check_update_banner() {
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

            let is_greater = self_update::version::bump_is_greater(current_ver, latest_ver).unwrap_or(false);

            if is_greater {
                println!(
                    "\n {}{}+-----------------------------------------------------------+{}",
                    SetAttribute(Attribute::Bold),
                    SetForegroundColor(Color::AnsiValue(214)),
                    ResetColor
                );
                println!(
                    " {}|  Notice: A new version of lshell is available ({:<10}) {}",
                    SetForegroundColor(Color::AnsiValue(214)),
                    latest.version,
                    ResetColor
                );
                println!(
                    " {}|  Run 'lshell updater' or 'updater' to update. {}",
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

pub fn builtin_update() -> i32 {
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

pub fn builtin_install_wt() -> i32 {
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

pub fn builtin_install_vscode() -> i32 {
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

pub fn builtin_install(args: &[String]) -> i32 {
    let sub = args.get(1).map(|s| s.to_lowercase());
    match sub.as_deref() {
        Some("wt") | Some("windows-terminal") => builtin_install_wt(),
        Some("vscode") | Some("code") => builtin_install_vscode(),
        _ => {
            let code_wt = builtin_install_wt();
            let code_vc = builtin_install_vscode();
            if code_wt == 0 || code_vc == 0 { 0 } else { 1 }
        }
    }
}

pub fn builtin_clear() -> i32 {
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
