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
| `edit <file>`| `ledit` | Launch the full-screen interactive TUI text editor. |
| `touch <file>`| | Create one or more empty files. |
| `mkdir <dir>`| | Create one or more directory paths recursively. |
| `rm <path>` | `del` | Remove files or directories recursively. |
| `export <K=V>`| | View or set environment variables (`export KEY=VALUE`). |
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
