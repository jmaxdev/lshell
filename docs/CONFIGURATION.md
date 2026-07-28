# Configuration & Themes Guide

`lshell` reads its configuration from `~/.lshell` in TOML format.

---

## ⚙️ Configuration File (`~/.lshell`)

Example configuration file:

```toml
theme = "minimal"               # Available themes: "minimal", "agnoster", "nord", "dracula"
prompt_symbol = "$ "
success_symbol = "○"
error_symbol = "✖"
show_git = true
show_time = true
use_powerline_symbols = true
enable_autosuggestions = true
min_autosuggestion_len = 2
enable_syntax_highlighting = true
show_dev_badge = true           # Enables dev environment badges (Rust, Node, Python, Docker, Go)
tree_max_depth = 3

[aliases]
ll = "ls -la"
g = "git"
cls = "clear"
e = "edit"
```

---

## 🎨 Themes

1. **`minimal`**: Clean single-line / double-line minimal prompt with colored status indicators.
2. **`agnoster`**: Powerline segment-based prompt displaying user, folder, git branch, and error code.
3. **`nord`**: Styled Powerline prompt using Nord color scheme (`#88C0D0`, `#81A1C1`).
4. **`dracula`**: Styled Powerline prompt using Dracula color scheme (Purple `#BD93F9`, Pink `#FF79C6`, Green `#50FA7B`).

---

## 🏷️ Developer Environment Badges

When `show_dev_badge = true` is enabled in `~/.lshell`, `lshell` detects project manifest files in the working directory and displays a badge in the prompt:

- 🦀 `Rust` (`Cargo.toml`)
- ⬢ `Node` (`package.json`)
- 🐍 `Python` (`pyproject.toml` / `requirements.txt`)
- 🐳 `Docker` (`Dockerfile` / `docker-compose.yml`)
- 🐹 `Go` (`go.mod`)
- ☕ `Java` (`pom.xml` / `build.gradle` / `build.gradle.kts`)
- 🐘 `PHP` (`composer.json`)
- 💎 `Ruby` (`Gemfile`)
- ⚡ `Zig` (`build.zig`)
- 🛠️ `C/C++` (`CMakeLists.txt` / `Makefile`)

---

## 📁 Storage Files

- **`~/.lshell`**: TOML configuration file and saved command aliases.
- **`~/.lshell_history`**: Plain-text history file preserving command history across sessions.
- **`~/.lshell_z`**: Database file storing visited directory frequency scores for `z` / `jump`.
