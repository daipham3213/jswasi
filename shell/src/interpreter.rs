use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use conch_parser::ast;

use crate::shell_base::{syscall, Redirect, Shell, EXIT_FAILURE, EXIT_SUCCESS};

pub fn interpret(shell: &mut Shell, cmd: &ast::TopLevelCommand<String>) {
    // println!("{:#?}", cmd);
    match &cmd.0 {
        ast::Command::Job(list) => handle_listable_command(shell, list, true),
        ast::Command::List(list) => handle_listable_command(shell, list, false),
    };
}

fn handle_listable_command(shell: &mut Shell, list: &ast::DefaultAndOrList, background: bool) {
    let mut status_code = match &list.first {
        ast::ListableCommand::Single(cmd) => {
            handle_pipeable_command(shell, cmd, background, &mut Vec::new())
        }
        ast::ListableCommand::Pipe(negate, cmds) => handle_pipe(shell, *negate, cmds, background),
    };

    for next_cmd in &list.rest {
        match (status_code, next_cmd) {
            (0, ast::AndOr::And(cmd)) => {
                status_code = match &cmd {
                    ast::ListableCommand::Single(cmd) => {
                        handle_pipeable_command(shell, cmd, background, &mut Vec::new())
                    }
                    ast::ListableCommand::Pipe(negate, cmds) => {
                        handle_pipe(shell, *negate, cmds, background)
                    }
                }
            }
            (x, ast::AndOr::Or(cmd)) if x != 0 => {
                status_code = match &cmd {
                    ast::ListableCommand::Single(cmd) => {
                        handle_pipeable_command(shell, cmd, background, &mut Vec::new())
                    }
                    ast::ListableCommand::Pipe(negate, cmds) => {
                        handle_pipe(shell, *negate, cmds, background)
                    }
                }
            }
            (_, _) => {}
        }
    }
}

fn handle_pipe(
    shell: &mut Shell,
    // TODO: handle negate
    negate: bool,
    cmds: &[ast::DefaultPipeableCommand],
    background: bool,
) -> i32 {
    handle_pipeable_command(
        shell,
        &cmds[0],
        background,
        // TODO: name of the virtual file should be uniquely generated
        // TODO: add virtual mode that won't create files but in-memory strings
        &mut vec![Redirect::Write((1u16, "/proc/pipe0.txt".to_string()))],
    );

    for (i, cmd) in cmds.iter().enumerate().skip(1).take(cmds.len() - 2) {
        handle_pipeable_command(
            shell,
            cmd,
            background,
            &mut vec![
                Redirect::Read((0u16, format!("/proc/pipe{}.txt", i - 1))),
                Redirect::Write((1u16, format!("/proc/pipe{}.txt", i))),
            ],
        );
    }

    let exit_status = handle_pipeable_command(
        shell,
        cmds.last().unwrap(),
        background,
        &mut vec![Redirect::Read((
            0u16,
            format!("/proc/pipe{}.txt", cmds.len() - 2),
        ))],
    );

    // if ! was present at the begining of the pipe, return logical negation of last command status
    if negate {
        (exit_status != 0) as i32
    } else {
        exit_status
    }
}

fn handle_pipeable_command(
    shell: &mut Shell,
    cmd: &ast::DefaultPipeableCommand,
    background: bool,
    redirects: &mut Vec<Redirect>,
) -> i32 {
    match cmd {
        ast::PipeableCommand::Simple(cmd) => {
            handle_simple_command(shell, cmd, background, redirects)
        }
        any => {
            println!("PipeableCommand not yet handled: {:#?}", any);
            EXIT_FAILURE
        }
    }
}

fn handle_simple_command(
    shell: &mut Shell,
    cmd: &ast::DefaultSimpleCommand,
    background: bool,
    redirects: &mut Vec<Redirect>,
) -> i32 {
    let env = cmd
        .redirects_or_env_vars
        .iter()
        .filter_map(|redirect_or_env_var| match redirect_or_env_var {
            ast::RedirectOrEnvVar::EnvVar(key, value) => {
                let value = match value {
                    None => Some("".to_string()),
                    Some(top_level_word) => handle_top_level_word(shell, top_level_word),
                };
                value.map(|value| (key.clone(), value))
            }
            _ => None,
        })
        .collect::<HashMap<_, _>>();

    let mut args = Vec::new();
    for redirect_or_cmd_word in &cmd.redirects_or_cmd_words {
        match redirect_or_cmd_word {
            ast::RedirectOrCmdWord::Redirect(redirect_type) => {
                if let Some(redirect) = handle_redirect_type(shell, redirect_type) {
                    redirects.push(redirect);
                }
            }
            ast::RedirectOrCmdWord::CmdWord(cmd_word) => {
                if let Some(arg) = handle_top_level_word(shell, &cmd_word.0) {
                    args.push(arg);
                }
            }
        }
    }

    if !args.is_empty() {
        match shell.execute_command(&args.remove(0), &mut args, &env, background, redirects) {
            Ok(result) => result,
            Err(error) => {
                println!("shell error: {:?}", error);
                EXIT_FAILURE
            }
        }
    } else {
        for (key, value) in env.iter() {
            // if it's a global update env, if shell variable update only vars
            if env::var(key).is_ok() {
                env::set_var(&key, &value);
                let _ = syscall("set_env", &[key, value], &env, false, redirects);
            } else {
                shell.vars.insert(key.clone(), value.clone());
            }
        }
        EXIT_SUCCESS
    }
}

fn handle_redirect_type(
    shell: &Shell,
    redirect_type: &ast::Redirect<ast::TopLevelWord<String>>,
) -> Option<Redirect> {
    match redirect_type {
        ast::Redirect::Write(file_descriptor, top_level_word) => {
            let file_descriptor = file_descriptor.unwrap_or(1);
            if let Some(mut filename) = handle_top_level_word(shell, top_level_word) {
                if !filename.starts_with('/') {
                    filename = PathBuf::from(&shell.pwd)
                        .join(&filename)
                        .display()
                        .to_string()
                }
                Some(Redirect::Write((file_descriptor, filename)))
            } else {
                None
            }
        }
        ast::Redirect::Append(file_descriptor, top_level_word) => {
            let file_descriptor = file_descriptor.unwrap_or(1);
            if let Some(mut filename) = handle_top_level_word(shell, top_level_word) {
                if !filename.starts_with('/') {
                    filename = PathBuf::from(&shell.pwd)
                        .join(&filename)
                        .display()
                        .to_string()
                }
                Some(Redirect::Append((file_descriptor, filename)))
            } else {
                None
            }
        }
        ast::Redirect::Read(file_descriptor, top_level_word) => {
            let file_descriptor = file_descriptor.unwrap_or(0);
            if let Some(mut filename) = handle_top_level_word(shell, top_level_word) {
                if !filename.starts_with('/') {
                    filename = PathBuf::from(&shell.pwd)
                        .join(&filename)
                        .display()
                        .to_string()
                }
                Some(Redirect::Read((file_descriptor, filename)))
            } else {
                None
            }
        }
        any => {
            println!("Redirect not yet handled: {:?}", any);
            None
        }
    }
}

fn handle_top_level_word<'a>(
    shell: &'a Shell,
    word: &'a ast::DefaultComplexWord,
) -> Option<String> {
    match word {
        ast::ComplexWord::Single(word) => handle_single(shell, word),
        ast::ComplexWord::Concat(words) => Some(
            words
                .iter()
                .filter_map(|w| handle_single(shell, w))
                .collect::<Vec<_>>()
                .join(""),
        ),
    }
}

fn handle_single<'a>(shell: &'a Shell, word: &'a ast::DefaultWord) -> Option<String> {
    match &word {
        ast::Word::SingleQuoted(w) => Some(w.clone()),
        ast::Word::Simple(w) => handle_simple_word(shell, w),
        ast::Word::DoubleQuoted(words) => Some(
            words
                .iter()
                .filter_map(|w| handle_simple_word(shell, w))
                .collect::<Vec<_>>()
                .join(" "),
        ),
    }
}

fn handle_simple_word<'a>(shell: &'a Shell, word: &'a ast::DefaultSimpleWord) -> Option<String> {
    match word {
        ast::SimpleWord::Literal(w) => Some(w.clone()),
        ast::SimpleWord::Colon => Some(":".to_string()),
        ast::SimpleWord::Tilde => Some(env::var("HOME").unwrap()),
        ast::SimpleWord::Param(p) => match p {
            ast::Parameter::Var(key) => {
                if let Some(variable) = shell.vars.get(key) {
                    Some(variable.clone())
                } else {
                    env::var(key).ok()
                }
            }
            ast::Parameter::Question => Some(shell.last_exit_status.to_string()),
            any => Some(format!("{:?}", any)),
        },
        any => Some(format!("{:?}", any)),
    }
}
