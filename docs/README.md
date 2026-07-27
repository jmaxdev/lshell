# lshell Documentation Index

Welcome to the documentation for **lshell** (Little Shell), a modular, high-performance command-line shell written in Rust.

---

## 📑 Documentation Modules

- **[Built-in Commands Reference](COMMANDS.md)**
  Complete specification of all built-in commands (`cd`, `z`, `ls`, `tree`, `search`, `usage`, `bench`, `top`, `sys`, `cat`, `edit`, `touch`, `mkdir`, `rm`, `export`, `which`, `history`, `alias`, `clear`, `help`, `exit`).

- **[Line Editor & Text Editor (ledit)](EDITOR.md)**
  Guide to the interactive line editor (`Ctrl+R` reverse search, auto-suggestions, `$VAR` expansion) and the built-in full-screen TUI text editor `ledit` (`Ctrl+F` find search, line numbers, cut/paste keybindings).

- **[Configuration & Themes](CONFIGURATION.md)**
  Configuration guide for `~/.lshell`, custom aliases, prompt themes (`minimal`, `agnoster`, `nord`, `dracula`), and developer environment badges.

- **[CLI Integrations & Standalone Tools](INTEGRATIONS.md)**
  Integration guide for Windows Terminal, VS Code integrated profiles, GitHub Releases updater, and the standalone `--prompt` engine.
