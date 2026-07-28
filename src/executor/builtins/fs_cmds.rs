use crossterm::style::{Attribute, Color, ResetColor, SetAttribute, SetForegroundColor};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub fn builtin_cd(prev_dir: &mut Option<PathBuf>, args: &[String]) -> i32 {
    let target = if args.is_empty() || args[0] == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
    } else if args[0] == "-" {
        if let Some(ref prev) = prev_dir {
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
            *prev_dir = current;
            if let Ok(cwd) = env::current_dir() {
                super::shell_cmds::record_z_visit(&cwd);
            }
            0
        }
        Err(e) => {
            eprintln!("lshell: cd: {}: {}", target.display(), e);
            1
        }
    }
}

pub fn builtin_pwd() -> i32 {
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

pub fn builtin_ls(args: &[String]) -> i32 {
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
            println!(
                " {:<2} {}{}{:<26}{} {}{}{}",
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
            println!(
                " {:<2} {}{:<26}{} {}{}{}",
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

pub fn builtin_tree(args: &[String]) -> i32 {
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

    let target =
        target_path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
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

pub fn builtin_cat(args: &[String]) -> i32 {
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

pub fn builtin_head(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("lshell: head: usage: head [-n <count>] <file>");
        return 1;
    }

    let mut count: usize = 10;
    let mut file_idx = 0;

    if args.len() >= 3 && (args[0] == "-n" || args[0] == "--lines") {
        if let Ok(c) = args[1].parse::<usize>() {
            count = c;
            file_idx = 2;
        }
    }

    if file_idx >= args.len() {
        eprintln!("lshell: head: missing file argument");
        return 1;
    }

    let path = PathBuf::from(&args[file_idx]);
    match fs::read_to_string(&path) {
        Ok(content) => {
            println!();
            for (idx, line) in content.lines().take(count).enumerate() {
                println!(
                    " {}{:4}{} | {}",
                    SetForegroundColor(Color::AnsiValue(242)),
                    idx + 1,
                    ResetColor,
                    line
                );
            }
            println!();
            0
        }
        Err(e) => {
            eprintln!("lshell: head: {}: {}", path.display(), e);
            1
        }
    }
}

pub fn builtin_tail(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("lshell: tail: usage: tail [-n <count>] <file>");
        return 1;
    }

    let mut count: usize = 10;
    let mut file_idx = 0;

    if args.len() >= 3 && (args[0] == "-n" || args[0] == "--lines") {
        if let Ok(c) = args[1].parse::<usize>() {
            count = c;
            file_idx = 2;
        }
    }

    if file_idx >= args.len() {
        eprintln!("lshell: tail: missing file argument");
        return 1;
    }

    let path = PathBuf::from(&args[file_idx]);
    match fs::read_to_string(&path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let skip_count = lines.len().saturating_sub(count);
            println!();
            for (idx, line) in lines.into_iter().skip(skip_count).enumerate() {
                println!(
                    " {}{:4}{} | {}",
                    SetForegroundColor(Color::AnsiValue(242)),
                    skip_count + idx + 1,
                    ResetColor,
                    line
                );
            }
            println!();
            0
        }
        Err(e) => {
            eprintln!("lshell: tail: {}: {}", path.display(), e);
            1
        }
    }
}

pub fn builtin_cp(args: &[String]) -> i32 {
    let filtered: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if filtered.len() < 2 {
        eprintln!("lshell: cp: usage: cp [-r] <src> <dest>");
        return 1;
    }

    let src = PathBuf::from(filtered[0]);
    let dest = PathBuf::from(filtered[1]);

    if !src.exists() {
        eprintln!(
            "lshell: cp: cannot stat '{}': No such file or directory",
            src.display()
        );
        return 1;
    }

    if src.is_dir() {
        let target = if dest.exists() && dest.is_dir() {
            dest.join(src.file_name().unwrap_or_default())
        } else {
            dest
        };
        if let Err(e) = copy_dir_all(&src, &target) {
            eprintln!(
                "lshell: cp: error copying directory '{}': {}",
                src.display(),
                e
            );
            return 1;
        }
    } else {
        let target = if dest.exists() && dest.is_dir() {
            dest.join(src.file_name().unwrap_or_default())
        } else {
            dest
        };
        if let Err(e) = fs::copy(&src, &target) {
            eprintln!("lshell: cp: error copying file '{}': {}", src.display(), e);
            return 1;
        }
    }
    0
}

pub fn builtin_mv(args: &[String]) -> i32 {
    let filtered: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    if filtered.len() < 2 {
        eprintln!("lshell: mv: usage: mv <src> <dest>");
        return 1;
    }

    let src = PathBuf::from(filtered[0]);
    let dest = PathBuf::from(filtered[1]);

    if !src.exists() {
        eprintln!(
            "lshell: mv: cannot stat '{}': No such file or directory",
            src.display()
        );
        return 1;
    }

    let target = if dest.exists() && dest.is_dir() {
        dest.join(src.file_name().unwrap_or_default())
    } else {
        dest
    };

    if fs::rename(&src, &target).is_err() {
        if src.is_dir() {
            if let Err(e) = copy_dir_all(&src, &target) {
                eprintln!("lshell: mv: {}", e);
                return 1;
            }
            let _ = fs::remove_dir_all(&src);
        } else {
            if let Err(e) = fs::copy(&src, &target) {
                eprintln!("lshell: mv: {}", e);
                return 1;
            }
            let _ = fs::remove_file(&src);
        }
    }
    0
}

pub fn builtin_touch(args: &[String]) -> i32 {
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

pub fn builtin_mkdir(args: &[String]) -> i32 {
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

pub fn builtin_rm(args: &[String]) -> i32 {
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

pub fn builtin_search(args: &[String]) -> i32 {
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

pub fn builtin_usage(args: &[String]) -> i32 {
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

    items.sort_by_key(|b| std::cmp::Reverse(b.1));

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

        items.sort_by_key(|a| a.file_name());

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

pub fn get_file_style(name: &str, is_dir: bool) -> (&'static str, Color, bool) {
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
    } else if lower.ends_with(".toml")
        || lower.ends_with(".json")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml")
    {
        ("\u{E615}", Color::AnsiValue(220), false)
    } else if lower.ends_with(".lock") {
        ("\u{F023}", Color::AnsiValue(245), false)
    } else if lower.starts_with('.') {
        ("\u{F418}", Color::AnsiValue(242), false)
    } else if lower.ends_with(".exe")
        || lower.ends_with(".bat")
        || lower.ends_with(".cmd")
        || lower.ends_with(".sh")
    {
        ("\u{F0E7}", Color::AnsiValue(78), true)
    } else if lower.ends_with(".md") || lower.ends_with(".txt") {
        ("\u{E609}", Color::AnsiValue(111), false)
    } else if lower.ends_with(".png")
        || lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".svg")
        || lower.ends_with(".ico")
    {
        ("\u{F03E}", Color::AnsiValue(177), false)
    } else if lower.ends_with(".zip")
        || lower.ends_with(".tar")
        || lower.ends_with(".gz")
        || lower.ends_with(".7z")
    {
        ("\u{F410}", Color::AnsiValue(172), false)
    } else {
        ("\u{F15B}", Color::AnsiValue(252), false)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
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

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}
