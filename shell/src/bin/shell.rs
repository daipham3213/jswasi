use std::env;

use clap::{App, Arg};
use std::path::PathBuf;

use shell::Shell;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let name = {
        let mut path = PathBuf::from(env::args().next().unwrap_or_else(|| "shell".to_string()));
        path.set_extension("");
        path.file_name().unwrap().to_str().unwrap().to_string()
    };
    let matches = App::new(name)
        .version(&*format!(
            "{}-{} ({})\nCopyright (c) 2021 Antmicro <www.antmicro.com>",
            env!("CARGO_PKG_VERSION"),
            env!("SHELL_COMMIT_HASH"),
            env!("SHELL_TARGET")
        ))
        .author("Antmicro <www.antmicro.com>")
        .arg(
            Arg::new("FILE")
                .about("Execute commands from file")
                .index(1),
        )
        .arg(
            Arg::new("command")
                .about("Execute provided command")
                .short('c')
                .long("command")
                .value_name("COMMAND")
                .takes_value(true),
        )
        .get_matches();

    if env::var("PWD").is_err() {
        env::set_var("PWD", "/");
    }
    if env::var("HOME").is_err() {
        env::set_var("HOME", "/");
    }
    let pwd = env::var("PWD")?;
    env::set_current_dir(&pwd)?;
    let mut shell = Shell::new(&pwd);

    if let Some(command) = matches.value_of("command") {
        shell.run_command(command)
    } else if let Some(file) = matches.value_of("FILE") {
        shell.run_script(file)
    } else {
        shell.run_interpreter()
    }
}