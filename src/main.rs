mod cli;
mod command;
mod engine;
mod i18n;
mod tui;

use clap::Parser;

use crate::cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();
    let json_output = cli.json;

    match cli.command {
        // completions 不需要文件参数
        Some(Command::Completions { shell }) => {
            use clap::CommandFactory;
            use clap_complete::generate;
            let mut cmd = Cli::command();
            generate(shell, &mut cmd, "jed", &mut std::io::stdout());
        }

        Some(cmd) => {
            let file = require_file(cli.file);
            command::run(&file, cmd, json_output);
        }

        None => {
            let file = require_file(cli.file);
            if let Err(e) = tui::run_tui(file) {
                let locale = i18n::get_locale();
                eprintln!("{}: {e}", i18n::t_to("main.tui_error", &locale));
                std::process::exit(1);
            }
        }
    }
}

fn require_file(file: Option<std::path::PathBuf>) -> std::path::PathBuf {
    file.unwrap_or_else(|| {
        let locale = i18n::get_locale();
        eprintln!("{}", i18n::t_to("main.need_file", &locale));
        std::process::exit(1);
    })
}
