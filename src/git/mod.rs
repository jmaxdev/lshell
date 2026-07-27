use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GitInfo {
    pub branch: String,
    pub is_dirty: bool,
    pub staged: u32,
    pub unstaged: u32,
    pub untracked: u32,
}

impl GitInfo {
    pub fn get(current_dir: &Path) -> Option<Self> {
        let git_dir = find_git_repo(current_dir)?;
        let head_path = git_dir.join("HEAD");

        if !head_path.exists() {
            return None;
        }

        let branch = match fs::read_to_string(&head_path) {
            Ok(content) => {
                let content = content.trim();
                if let Some(stripped) = content.strip_prefix("ref: refs/heads/") {
                    stripped.to_string()
                } else if content.len() >= 7 {
                    content[..7].to_string()
                } else {
                    "HEAD".to_string()
                }
            }
            Err(_) => return None,
        };

        let (is_dirty, staged, unstaged, untracked) = get_git_status_counts(current_dir);

        Some(GitInfo {
            branch,
            is_dirty,
            staged,
            unstaged,
            untracked,
        })
    }
}

fn find_git_repo(start_dir: &Path) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();
    loop {
        let git_path = current.join(".git");
        if git_path.exists() {
            return Some(git_path);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

fn get_git_status_counts(dir: &Path) -> (bool, u32, u32, u32) {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(dir)
        .output();

    let mut staged = 0;
    let mut unstaged = 0;
    let mut untracked = 0;

    if let Ok(out) = output {
        if out.status.success() {
            let stdout = String::from_utf8_lossy(&out.stdout);
            for line in stdout.lines() {
                if line.len() < 2 {
                    continue;
                }
                let bytes = line.as_bytes();
                let x = bytes[0] as char;
                let y = bytes[1] as char;

                if x == '?' && y == '?' {
                    untracked += 1;
                } else {
                    if x != ' ' && x != '?' {
                        staged += 1;
                    }
                    if y != ' ' && y != '?' {
                        unstaged += 1;
                    }
                }
            }
        }
    }

    let is_dirty = staged > 0 || unstaged > 0 || untracked > 0;
    (is_dirty, staged, unstaged, untracked)
}
