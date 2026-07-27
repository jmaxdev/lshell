mod config;
mod editor;
mod executor;
mod git;
mod ledit;
mod prompt;
mod theme;

use config::Config;
use editor::LineEditor;
use executor::Executor;
use prompt::PromptBuilder;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::load();

    if args.len() >= 2 {
        let arg = args[1].as_str();
        if arg == "updater" || arg == "update" || arg == "--update" {
            let executor = Executor::new();
            executor.builtin_update();
            return;
        } else if arg == "install-wt" || arg == "wt-install" || arg == "--install-wt" {
            let executor = Executor::new();
            executor.builtin_install_wt();
            return;
        } else if arg == "install-vscode" || arg == "vscode-install" || arg == "--install-vscode" {
            let executor = Executor::new();
            executor.builtin_install_vscode();
            return;
        } else if arg == "install" || arg == "--install" {
            let executor = Executor::new();
            let sub = args.get(2).map(|s| s.to_lowercase());
            match sub.as_deref() {
                Some("wt") | Some("windows-terminal") => {
                    executor.builtin_install_wt();
                }
                Some("vscode") | Some("code") => {
                    executor.builtin_install_vscode();
                }
                _ => {
                    executor.builtin_install_wt();
                    executor.builtin_install_vscode();
                }
            }
            return;
        } else if arg == "--prompt" {
            let code = args.get(2).and_then(|s| s.parse::<i32>().ok()).unwrap_or(0);
            let prompt_str = PromptBuilder::build(code, &config);
            print!("{}", prompt_str);
            return;
        }
    }

    let mut config = Config::load();
    Config::save_default();

    let mut executor = Executor::new();
    executor.check_update_banner();
    let mut last_exit_code = 0;

    loop {
        let prompt_str = PromptBuilder::build(last_exit_code, &config);

        match LineEditor::read_line(&prompt_str, executor.get_history(), &config) {
            Ok(input) => {
                let trimmed = input.trim();
                if trimmed.to_lowercase() == "exit" || trimmed.to_lowercase() == "quit" {
                    println!("Goodbye!");
                    break;
                }

                last_exit_code = executor.execute(&input, &mut config);
            }
            Err(err) => {
                eprintln!("\nTerminal error: {}", err);
                break;
            }
        }
    }
}
