# Little Shell

## Description

lshell is a modular, high-performance command-line shell written in Rust. It provides an advanced interactive terminal interface featuring history-based auto-suggestions, real-time syntax highlighting, native Git integration, customizable themes, an automated binary updater, and built-in TUI utilities such as a full-screen text editor.

## Features

- Real-time syntax highlighting for valid and invalid commands.
- Intelligent auto-suggestions powered by command history.
- Interactive history search via Ctrl+R.
- Native Git repository integration displaying active branch and dirty status.
- Integrated full-screen text editor (Little Edit).
- Built-in directory tree visualizer.
- Centralized configuration system in TOML format (`~/.lshell`).
- Custom user-defined command aliases.
- Startup notification banner advising when a new release is available.
- Interactive and CLI updater via `lshell updater` or `updater` (GitHub Releases with Git fallback).
- Standalone prompt generation mode via the `--prompt` flag.

## Requirements (Building from Source)

- Rust 1.80.0 or higher.
- Cargo (Rust package manager).

### Compilation and Installation

To build the project in optimized production mode:

```bash
cargo build --release
```

The compiled binary will be located at:

```text
target/release/lshell
```

To install the executable binary to your local bin path (Linux / macOS):

```bash
cp target/release/lshell ~/.local/bin/
```

Or on Windows (PowerShell):

```powershell
Copy-Item target\release\lshell.exe $HOME\AppData\Local\Microsoft\WindowsApps\
```

## Usage Examples

### Interactive Shell Navigation and Inspection

Navigating directories using quick navigation shortcuts:

```text
lshell> cd ..       # Move up one directory level
lshell> cd ...      # Move up two directory levels
lshell> cd ....     # Move up three directory levels
lshell> cd -        # Switch to previous working directory (OLDPWD)
```

Listing directories, viewing trees, and reading file contents:

```text
lshell> ls -la
lshell> tree src/
lshell> cat src/main.rs
```

Checking system info, environment variables, and executable paths:

```text
lshell> sys
lshell> export PORT=8080
lshell> which git
```

### Integrated Text Editor (Little Edit)

Launch the interactive full-screen TUI text editor:

```text
lshell> edit main.rs
```

Keybindings inside Little Edit:
- `Ctrl+O` or `Ctrl+S`: Save file.
- `Ctrl+K`: Cut current line.
- `Ctrl+U`: Paste cut line.
- `Ctrl+C`: Display line number, column, and total character status.
- `Ctrl+X`: Exit editor (prompts confirmation if file was modified).

### Interactive History Search

Press `Ctrl+R` to search backward through your command history. Type a search query to filter commands dynamically. Press `Enter` or `Esc` to load the selected command into the prompt buffer.

### Running the Updater

Check for and install updates from GitHub Releases:

```bash
# Executing updater directly from terminal CLI
$ lshell updater

# Executing updater from within an active lshell session
lshell> updater
```

## Integration Examples

### 1. Windows Terminal Integration

You can automatically register `lshell` into Windows Terminal's `settings.json` profile list using the built-in `install-wt` command:

```bash
# Register from outside lshell CLI
$ lshell install wt

# Or register from inside an active lshell session
lshell> install wt
```

Alternatively, you can manually add `lshell` to your Windows Terminal profile settings (`settings.json`):

```json
{
    "profiles": {
        "list": [
            {
                "guid": "{a6c8e547-817d-4c3e-96a8-f3d99e098711}",
                "name": "lshell",
                "commandline": "C:\\path\\to\\lshell.exe",
                "startingDirectory": "%USERPROFILE%"
            }
        ]
    }
}
```

### 2. VS Code Terminal Integration

You can automatically set `lshell` as your default integrated terminal profile in Visual Studio Code using the `install-vscode` command (or run `install` to configure both Windows Terminal and VS Code):

```bash
# Register VS Code profile from CLI
$ lshell install vscode

# Or register inside an active lshell session
lshell> install vscode

# Or register into both Windows Terminal and VS Code at once
$ lshell install
```

Alternatively, you can manually set `lshell` as your default integrated terminal profile in Visual Studio Code (`.vscode/settings.json` or User `settings.json`):

```json
{
    "terminal.integrated.profiles.windows": {
        "lshell": {
            "path": "C:\\path\\to\\lshell.exe",
            "icon": "terminal-bash"
        }
    },
    "terminal.integrated.defaultProfile.windows": "lshell"
}
```

For Linux or macOS:

```json
{
    "terminal.integrated.profiles.linux": {
        "lshell": {
            "path": "/usr/local/bin/lshell"
        }
    },
    "terminal.integrated.defaultProfile.linux": "lshell"
}
```

### 3. Standalone Prompt Engine Integration (`--prompt`)

`lshell` features an independent prompt renderer that can be invoked by external shells or scripts. Pass `--prompt <exit_code>` to output the rendered prompt string based on your `~/.lshell` configuration:

```bash
# Render prompt with exit status 0 (success)
$ lshell --prompt 0

# Render prompt with exit status 1 (error)
$ lshell --prompt 1
```

Integration into Zsh (`~/.zshrc`):

```zsh
# Consume lshell prompt engine inside Zsh
PROMPT='$(lshell --prompt $?)'
```

Integration into Bash (`~/.bashrc`):

```bash
# Consume lshell prompt engine inside Bash
PROMPT_COMMAND='PS1="$(lshell --prompt $?)"'
```

### 4. Custom Configuration & Aliases Integration (`~/.lshell`)

Customize themes, symbols, feature toggles, and command aliases in `~/.lshell`:

```toml
theme = "minimal"
prompt_symbol = "$ "
success_symbol = "○"
error_symbol = "✖"
show_git = true
show_time = true
use_powerline_symbols = true
enable_autosuggestions = true
min_autosuggestion_len = 2
enable_syntax_highlighting = true
tree_max_depth = 3

[aliases]
ll = "ls -la"
g = "git"
cls = "clear"
e = "edit"
gs = "git status"
gp = "git pull"
```

## Built-in Commands

- `cd <directory>`: File system navigation (supports `..`, `...`, `....`, and `-`).
- `pwd`: Print current working directory.
- `ls` / `dir`: Formatted directory listing with file sizes and styling.
- `tree [directory]`: Display directory structure in a tree hierarchy.
- `cat` / `type <file>`: Print file contents with line numbers.
- `touch <file>`: Create an empty file.
- `mkdir <directory>`: Create directories.
- `rm` / `del <path>`: Remove files or directories recursively.
- `edit` / `ledit <file>`: Full-screen interactive text editor.
- `sys` / `info`: Display operating system, architecture, and environment details.
- `updater` / `update`: Check and install the latest binary release from GitHub Releases (or pull git updates).
- `install-wt` / `wt-install`: Automatically register lshell into Windows Terminal settings.json.
- `install-vscode` / `vscode-install`: Automatically register lshell as default terminal in VS Code settings.json.
- `install`: Automatically register lshell into both Windows Terminal and VS Code.
- `history`: Display executed command history.
- `export <VAR=VALUE>`: Set environment variables.
- `which` / `where <command>`: Locate binary executable or confirm built-in status.
- `clear` / `cls`: Clear terminal screen buffer.
- `help`: Display help menu and list of available built-in commands.
- `exit` / `quit`: Exit shell.

## Continuous Integration and Release

Automated workflows are managed via GitHub Actions:
- **CI Workflow** (`.github/workflows/ci.yml`): Runs cross-platform checks and builds on Linux, Windows, and macOS.
- **Release Workflow** (`.github/workflows/release.yml`): Automatically compiles release binaries upon tag pushes (`v*`) and publishes assets to GitHub Releases.

## License

This project is licensed under the terms of the UnSetSoft Public License 1.0 (UPL-1.0). For more details, refer to the LICENSE file in the repository root.
