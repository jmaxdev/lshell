# Line Editor & Text Editor (`ledit`)

`lshell` features an interactive REPL line editor and an integrated full-screen TUI text editor (`ledit`).

---

## 1. REPL Line Editor Features

- **Auto-Suggestions**: Displays faint history-based completions as you type. Press `Right Arrow` or `End` to accept completion.
- **Syntax Highlighting**: Real-time coloring for valid vs invalid commands and string parameters.
- **Reverse History Search (`Ctrl+R`)**: Press `Ctrl+R`, type a query to filter past commands dynamically, and press `Enter` or `Esc` to place it into the prompt buffer.
- **Environment Variable Expansion**: Variables starting with `$` (e.g. `$HOME`, `$USER`, `$PATH`) are expanded automatically before command execution.

---

## 2. Integrated Text Editor (`ledit`)

Launch `ledit` by running `edit <filename>` (or `ledit`).

### Features & UI
- **Gutter Line Numbers**: Displays line numbers (`0001 │ `) on the left margin.
- **Header & Status Bar**: Displays file name, modified status tag (`Modified`), line/column location, and status messages.
- **Syntax Highlighting**: Real-time token highlighting for keywords (`fn`, `pub`, `let`, `struct`, `impl`, `use`, `match`, `return`, `if`, `else`, `const`, `import`, `export`, etc.), comments (`//`, `#`), and string literals.

### Keybindings Cheat-Sheet

| Keybinding | Action |
| :--- | :--- |
| `Ctrl+S` / `Ctrl+O` | Save modified buffer to disk. |
| `Ctrl+F` / `Ctrl+W` | Open find text search bar. Press `Enter` to jump to next match, `Esc` to cancel. |
| `Ctrl+K` | Cut current line into internal clipboard. |
| `Ctrl+U` | Paste clipboard line into buffer at cursor. |
| `Ctrl+C` | Display detailed cursor location (line, column, character count). |
| `Ctrl+X` | Exit editor. Asks confirmation (`Y`/`N`) if buffer was modified. |
