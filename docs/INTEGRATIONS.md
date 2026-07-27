# Integrations & CLI Options

`lshell` provides native CLI flags and automated integrations with popular terminal multiplexers and IDEs.

---

## 1. Automated IDE & Terminal Registration

### Windows Terminal Integration
Automatically register `lshell` into Windows Terminal `settings.json` profiles:

```bash
# From command line
$ lshell install wt

# Inside interactive session
lshell> install-wt
```

### VS Code Terminal Integration
Automatically register `lshell` as the default integrated terminal in Visual Studio Code:

```bash
# From command line
$ lshell install vscode

# Inside interactive session
lshell> install-vscode

# Register into both Windows Terminal and VS Code at once
$ lshell install
```

---

## 2. Standalone Prompt Engine (`--prompt`)

`lshell` features an independent prompt renderer that can be invoked by external shell scripts. Pass `--prompt <exit_code>` to render the prompt string based on your `~/.lshell` config:

```bash
# Render prompt string with exit status 0 (success)
$ lshell --prompt 0

# Render prompt string with exit status 1 (error)
$ lshell --prompt 1
```

### Zsh Integration (`~/.zshrc`)

```zsh
PROMPT='$(lshell --prompt $?)'
```

### Bash Integration (`~/.bashrc`)

```bash
PROMPT_COMMAND='PS1="$(lshell --prompt $?)"'
```

---

## 3. Self-Updater (`updater`)

`lshell` includes an integrated updater that checks GitHub Releases for new binary assets and self-updates:

```bash
# From CLI
$ lshell updater

# Inside lshell session
lshell> update
```
