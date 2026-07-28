use super::parser::{CommandSpec, Pipeline, RedirectionKind};
use crate::config::Config;
use crate::executor::builtins::{fs_cmds, shell_cmds, sys_cmds};
use std::fs::{File, OpenOptions};
use std::io::{Read as IoRead, Write as IoWrite};
use std::path::PathBuf;
use std::process::{Command, Stdio};

pub fn run_pipeline(
    pipeline: &Pipeline,
    prev_dir: &mut Option<PathBuf>,
    history: &mut Vec<String>,
    config: &mut Config,
    bench_fn: impl Fn(&mut Vec<String>, Option<&mut PathBuf>, &mut Config, &str) -> i32,
) -> i32 {
    if pipeline.commands.is_empty() {
        return 0;
    }

    if pipeline.commands.len() == 1 {
        let cmd_spec = &pipeline.commands[0];
        return run_single_command(cmd_spec, prev_dir, history, config, bench_fn);
    }

    // Multi-stage pipeline execution (cmd1 | cmd2 | ...)
    let mut previous_stdout: Option<Vec<u8>> = None;
    let mut last_exit_code = 0;

    for (i, cmd_spec) in pipeline.commands.iter().enumerate() {
        let is_first = i == 0;
        let is_last = i == pipeline.commands.len() - 1;

        if cmd_spec.args.is_empty() {
            continue;
        }

        let cmd_name = cmd_spec.args[0].to_lowercase();
        let args = &cmd_spec.args[1..];

        if is_builtin(&cmd_name) {
            // Builtin in pipeline
            let code = run_builtin(&cmd_name, args, prev_dir, history, config);
            last_exit_code = code;
        } else {
            // External command in pipeline
            let mut command = Command::new(&cmd_spec.args[0]);
            command.args(args);

            if !is_first && previous_stdout.is_some() {
                command.stdin(Stdio::piped());
            }

            if !is_last {
                command.stdout(Stdio::piped());
            }

            match command.spawn() {
                Ok(mut child) => {
                    if !is_first {
                        if let Some(input_data) = previous_stdout.take() {
                            if let Some(mut stdin) = child.stdin.take() {
                                let _ = stdin.write_all(&input_data);
                            }
                        }
                    }

                    if !is_last {
                        if let Some(mut stdout) = child.stdout.take() {
                            let mut buffer = Vec::new();
                            let _ = stdout.read_to_end(&mut buffer);
                            previous_stdout = Some(buffer);
                        }
                    }

                    match child.wait() {
                        Ok(status) => {
                            last_exit_code = status.code().unwrap_or(0);
                        }
                        Err(e) => {
                            eprintln!("lshell: pipeline error: {}", e);
                            return 1;
                        }
                    }
                }
                Err(_) => {
                    #[cfg(windows)]
                    {
                        let mut cmd_shell = Command::new("cmd.exe");
                        cmd_shell.arg("/C").arg(&cmd_spec.args[0]).args(args);
                        if !is_last {
                            cmd_shell.stdout(Stdio::piped());
                        }
                        if let Ok(mut child) = cmd_shell.spawn() {
                            if !is_last {
                                if let Some(mut stdout) = child.stdout.take() {
                                    let mut buffer = Vec::new();
                                    let _ = stdout.read_to_end(&mut buffer);
                                    previous_stdout = Some(buffer);
                                }
                            }
                            if let Ok(status) = child.wait() {
                                last_exit_code = status.code().unwrap_or(0);
                                continue;
                            }
                        }
                    }
                    eprintln!("lshell: command not found: {}", cmd_spec.args[0]);
                    return 127;
                }
            }
        }
    }

    last_exit_code
}

fn run_single_command(
    cmd_spec: &CommandSpec,
    prev_dir: &mut Option<PathBuf>,
    history: &mut Vec<String>,
    config: &mut Config,
    bench_fn: impl Fn(&mut Vec<String>, Option<&mut PathBuf>, &mut Config, &str) -> i32,
) -> i32 {
    if cmd_spec.args.is_empty() {
        return 0;
    }

    let cmd_name = cmd_spec.args[0].to_lowercase();
    let args = &cmd_spec.args[1..];

    // Handle redirections
    let mut redir_out_file: Option<File> = None;
    let mut redir_in_file: Option<File> = None;

    for redir in &cmd_spec.redirections {
        match redir.kind {
            RedirectionKind::Write => {
                match File::create(&redir.target) {
                    Ok(f) => redir_out_file = Some(f),
                    Err(e) => {
                        eprintln!("lshell: redirection error (write): {}: {}", redir.target, e);
                        return 1;
                    }
                }
            }
            RedirectionKind::Append => {
                match OpenOptions::new().create(true).append(true).open(&redir.target) {
                    Ok(f) => redir_out_file = Some(f),
                    Err(e) => {
                        eprintln!("lshell: redirection error (append): {}: {}", redir.target, e);
                        return 1;
                    }
                }
            }
            RedirectionKind::Read => {
                match File::open(&redir.target) {
                    Ok(f) => redir_in_file = Some(f),
                    Err(e) => {
                        eprintln!("lshell: redirection error (read): {}: {}", redir.target, e);
                        return 1;
                    }
                }
            }
        }
    }

    if cmd_name == "bench" || cmd_name == "time" {
        let target_cmd = args.join(" ");
        return bench_fn(history, prev_dir.as_mut(), config, &target_cmd);
    }

    if is_builtin(&cmd_name) {
        return run_builtin(&cmd_name, args, prev_dir, history, config);
    }

    // External command execution
    let mut command = Command::new(&cmd_spec.args[0]);
    command.args(args);

    if let Some(file) = redir_out_file {
        command.stdout(Stdio::from(file));
    }
    if let Some(file) = redir_in_file {
        command.stdin(Stdio::from(file));
    }

    match command.status() {
        Ok(status) => status.code().unwrap_or(0),
        Err(e) => {
            #[cfg(windows)]
            {
                let mut cmd_shell = Command::new("cmd.exe");
                cmd_shell.arg("/C").arg(&cmd_spec.args[0]).args(args);
                if let Ok(status) = cmd_shell.status() {
                    return status.code().unwrap_or(0);
                }
            }
            eprintln!("lshell: command not found: {} ({})", cmd_spec.args[0], e);
            127
        }
    }
}

pub fn is_builtin(cmd: &str) -> bool {
    matches!(
        cmd,
        "cd" | "z"
            | "jump"
            | "alias"
            | "search"
            | "find"
            | "usage"
            | "du"
            | "top"
            | "ps"
            | "pwd"
            | "clear"
            | "cls"
            | "ls"
            | "dir"
            | "tree"
            | "sys"
            | "info"
            | "version"
            | "--version"
            | "-v"
            | "update"
            | "updater"
            | "install-wt"
            | "wt-install"
            | "install-vscode"
            | "vscode-install"
            | "install"
            | "cat"
            | "type"
            | "edit"
            | "ledit"
            | "touch"
            | "mkdir"
            | "rm"
            | "del"
            | "which"
            | "where"
            | "history"
            | "help"
            | "export"
            | "env"
            | "unset"
            | "head"
            | "tail"
            | "cp"
            | "copy"
            | "mv"
            | "move"
            | "exit"
            | "quit"
            | ".."
            | "..."
            | "...."
    )
}

pub fn run_builtin(
    cmd: &str,
    args: &[String],
    prev_dir: &mut Option<PathBuf>,
    history: &mut Vec<String>,
    config: &mut Config,
) -> i32 {
    match cmd {
        "exit" | "quit" => std::process::exit(0),
        "cd" => fs_cmds::builtin_cd(prev_dir, args),
        ".." => fs_cmds::builtin_cd(prev_dir, &["..".to_string()]),
        "..." => fs_cmds::builtin_cd(prev_dir, &["../..".to_string()]),
        "...." => fs_cmds::builtin_cd(prev_dir, &["../../..".to_string()]),
        "z" | "jump" => shell_cmds::builtin_z(prev_dir, args),
        "alias" => shell_cmds::builtin_alias(args, config),
        "search" | "find" => fs_cmds::builtin_search(args),
        "usage" | "du" => fs_cmds::builtin_usage(args),
        "top" | "ps" => sys_cmds::builtin_top(),
        "pwd" => fs_cmds::builtin_pwd(),
        "clear" | "cls" => sys_cmds::builtin_clear(),
        "ls" | "dir" => fs_cmds::builtin_ls(args),
        "tree" => fs_cmds::builtin_tree(args),
        "sys" | "info" => sys_cmds::builtin_sys(),
        "version" | "--version" | "-v" => sys_cmds::builtin_version(),
        "update" | "updater" => sys_cmds::builtin_update(),
        "install-wt" | "wt-install" => sys_cmds::builtin_install_wt(),
        "install-vscode" | "vscode-install" => sys_cmds::builtin_install_vscode(),
        "install" => sys_cmds::builtin_install(args),
        "cat" | "type" => fs_cmds::builtin_cat(args),
        "edit" | "ledit" => shell_cmds::builtin_edit(args),
        "touch" => fs_cmds::builtin_touch(args),
        "mkdir" => fs_cmds::builtin_mkdir(args),
        "rm" | "del" => fs_cmds::builtin_rm(args),
        "which" | "where" => shell_cmds::builtin_which(args),
        "history" => shell_cmds::builtin_history(history, args),
        "help" => shell_cmds::builtin_help(),
        "export" => shell_cmds::builtin_export(args),
        "env" => shell_cmds::builtin_env(args),
        "unset" => shell_cmds::builtin_unset(args),
        "head" => fs_cmds::builtin_head(args),
        "tail" => fs_cmds::builtin_tail(args),
        "cp" | "copy" => fs_cmds::builtin_cp(args),
        "mv" | "move" => fs_cmds::builtin_mv(args),
        _ => 1,
    }
}
