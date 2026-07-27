# Little Shell (lshell)

lshell is a modular, high-performance command-line shell written in Rust featuring interactive auto-suggestions, syntax highlighting, native Git integration, a built-in TUI text editor (`ledit`), and customizable themes.

## Documentation

Exhaustive documentation is modularized inside the `docs/` directory:

- [Documentation Index](docs/README.md)
- [Built-in Commands Reference](docs/COMMANDS.md)
- [Line Editor & Text Editor (ledit)](docs/EDITOR.md)
- [Configuration & Themes Guide](docs/CONFIGURATION.md)
- [Integrations & CLI Options](docs/INTEGRATIONS.md)

## Quick Start

### Build from source

```bash
cargo build --release
```

The compiled binary will be generated at `target/release/lshell`.

## License

Licensed under the [UnSetSoft Public License 1.0 (UPL-1.0)](LICENSE.md).
