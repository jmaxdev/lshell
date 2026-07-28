# Built-in Commands Reference

`lshell` includes a rich set of native built-in commands that execute directly inside the shell process for maximum speed and enhanced visual output.

---

## Command Table

| Command | Aliases | Description |
| :--- | :--- | :--- |
| `cd [dir]` | `..`, `...`, `....` | Navigate directories. `..` moves up 1 level, `...` up 2 levels, `....` up 3 levels, `-` switches to `OLDPWD`, `~` goes to home. |
| `z [query]` | `jump` | Smart directory jump based on visit history (`~/.lshell_z`). Running `z` without arguments lists top visited directories. |
| `ls [dir]` | `dir` | List files with vector icons, color coding, and file sizes. Use `-a` to include hidden files. |
| `tree [dir]` | | Display directory tree hierarchy. Options: `--full` for deep recursive tree (depth 3), `--file` to save to `tree.txt`. |
| `pwd` | | Print current working directory path. |
| `alias [k=v]` | | List active aliases or define a new alias. Use `alias --save` to persist aliases to `~/.lshell`. |
| `search <q>` | `find` | Recursively search for matching filenames and text line contents in target directory. |
| `usage [dir]` | `du` | Calculate disk usage and display horizontal ASCII percentage progress bars. |
| `bench <cmd>`| `time` | High-precision execution timer measuring command duration in milliseconds and reporting status. |
| `top` | `ps` | Display styled system information, OS, CPU cores, current PID, and memory model card. |
| `sys` | `info` | Display stylized system information card. |
| `version` | `--version`, `-v` | Print lshell version. |
| `cat <file>` | `type` | Print line-numbered contents of a text file. |
| `head <file>`| | Print first N lines of a text file (default 10, option `-n N`). |
| `tail <file>`| | Print last N lines of a text file (default 10, option `-n N`). |
| `cp <src> <dst>`| `copy` | Copy files or directories recursively. |
| `mv <src> <dst>`| `move` | Move or rename files or directories. |
| `edit <file>`| `ledit` | Launch the full-screen interactive TUI text editor. |
| `touch <file>`| | Create one or more empty files. |
| `mkdir <dir>`| | Create one or more directory paths recursively. |
| `rm <path>` | `del` | Remove files or directories recursively. |
| `export <K=V>`| | View or set environment variables (`export KEY=VALUE`). |
| `env [query]` | | View or filter active environment variables. |
| `unset <VAR>`| | Remove one or more environment variables. |
| `which <cmd>`| `where` | Locate binary executable path in `PATH` or confirm built-in status. |
| `history` | | Print indexed list of executed command history. |
| `update` | `updater` | Check for and install the latest lshell release from GitHub Releases. |
| `install` | | Register lshell into Windows Terminal (`install-wt`) and VS Code (`install-vscode`). |
| `clear` | `cls` | Clear terminal screen and scrollback buffer. |
| `help` | | Display the built-in command summary menu. |
| `exit` | `quit` | End lshell terminal session. |

---

## Detailed Command Examples

### Directory Tree Visualizer (`tree`)

```bash
# View level-1 tree of current directory
lshell> tree

# View full recursive tree (depth 3)
lshell> tree --full

# Save level-1 tree to tree.txt
lshell> tree --file

# Save full tree of src directory to tree.txt
lshell> tree src --full --file
```

### History Management (`history`)

```bash
# View command history list with indices
lshell> history

# Clear command history memory and delete ~/.lshell_history file
lshell> history clean
lshell> history clear
lshell> history -c
```

### Smart Directory Jump (`z`)

```bash
# View most frequently visited directories
lshell> z

# Jump to top directory matching "test"
lshell> z test
```

### Recursive Content Search (`search`)

```bash
# Search for string "fn main" in current directory
lshell> search "fn main"

# Search inside src directory
lshell> search "Config" src
```

### Disk Usage Meter (`usage`)

```bash
# Analyze disk usage of current directory
lshell> usage
```

### Command Benchmark Timer (`bench`)

```bash
# Benchmark cargo check duration
lshell> bench cargo check
```

---

## 🔀 Pipelines, Operators & Redirections

`lshell` supports native shell pipeline chained execution, logical operators, and file redirections:

| Feature | Syntax | Example | Description |
| :--- | :--- | :--- | :--- |
| **Pipeline** | `cmd1 \| cmd2` | `ls \| search Cargo` | Connects stdout of `cmd1` to stdin of `cmd2`. |
| **Logical AND** | `cmd1 && cmd2` | `cargo check && cargo run` | Executes `cmd2` only if `cmd1` succeeds (exit code 0). |
| **Logical OR** | `cmd1 \|\| cmd2` | `cargo test \|\| echo "Tests failed"` | Executes `cmd2` only if `cmd1` fails (non-zero exit code). |
| **Sequential** | `cmd1 ; cmd2` | `cls ; sys` | Executes `cmd1` followed by `cmd2` sequentially. |
| **Output Write** | `cmd > file` | `tree > tree_out.txt` | Overwrites `file` with the stdout of `cmd`. |
| **Output Append** | `cmd >> file` | `echo "log entry" >> log.txt` | Appends stdout of `cmd` to `file`. |
| **Input Read** | `cmd < file` | `cat < input.txt` | Redirects `file` contents into stdin of `cmd`. |

