mod cli;
mod command;
mod engine;
mod i18n;
mod tui;

use clap::{CommandFactory, Parser};
use std::io::Read;
use std::path::PathBuf;

use crate::cli::{resolve_file, Cli, Command};

fn main() {
    let cli = Cli::parse();

    // 设置语言环境
    if let Some(lang) = &cli.lang {
        // SAFETY: 单线程 CLI 应用中设置环境变量是安全的
        unsafe { std::env::set_var("JE_LANG", lang) };
    }

    let json_output = cli.json;

    match &cli.command {
        // 显示帮助
        _ if cli.help > 0 => {
            if cli.help > 1 || cli.command.is_none() {
                // -h 或没有子命令时显示全局帮助
                let mut cmd = Cli::command();
                cmd.print_help().ok();
                println!();
            } else if let Some(_cmd) = &cli.command {
                // 显示子命令帮助
                let mut cmd = Cli::command();
                cmd.print_help().ok();
            }
        }

        // 列出所有命令
        Some(Command::Commands {}) => {
            print_commands(json_output);
        }

        // 命令帮助
        Some(Command::Explain { command }) => {
            print_command_help(command, json_output);
        }

        // 补全脚本
        Some(Command::Completions { shell }) => {
            use clap::CommandFactory;
            use clap_complete::generate;
            let mut cmd = Cli::command();
            generate(*shell, &mut cmd, "jed", &mut std::io::stdout());
        }

        // Tree 命令
        Some(Command::Tree {
            file,
            expand_all,
            path,
        }) => {
            let f = resolve_file(cli.get_file().as_ref(), Some(file));
            let f = resolve_input_file(f.as_ref());
            command::run_tree(&f, *expand_all, path.as_deref(), json_output);
        }

        // Query 命令
        Some(Command::Query { filter, file }) => {
            let f = resolve_file(cli.get_file().as_ref(), Some(file));
            let f = resolve_input_file(f.as_ref());
            command::run_query(&f, filter, json_output);
        }

        // Validate 命令
        Some(Command::Validate { schema, file }) => {
            let f = resolve_file(cli.get_file().as_ref(), Some(file));
            let f = resolve_input_file(f.as_ref());
            command::run_validate(&f, schema, json_output);
        }

        // Convert 命令
        Some(Command::Convert { format, file }) => {
            let f = resolve_file(cli.get_file().as_ref(), Some(file));
            let f = resolve_input_file(f.as_ref());
            command::run_convert(&f, format, json_output);
        }

        Some(cmd) => {
            // 获取文件参数
            let file = get_file_from_command(cmd, cli.get_file().as_ref());
            let file = resolve_input_file(Some(&file));
            command::run(&file, cmd.clone(), json_output);
        }

        None => {
            // 无命令时进入 TUI 模式
            let file = cli.get_file().map(|f| {
                if f.to_str() == Some("-") {
                    // TUI 模式不支持 stdin
                    let locale = i18n::get_locale();
                    eprintln!("{}", i18n::t_to("main.need_file", &locale));
                    std::process::exit(1);
                }
                f.clone()
            });

            if let Some(file) = file {
                if let Err(e) = tui::run_tui(file) {
                    let locale = i18n::get_locale();
                    eprintln!("{}: {e}", i18n::t_to("main.tui_error", &locale));
                    std::process::exit(1);
                }
            } else {
                let locale = i18n::get_locale();
                eprintln!("{}", i18n::t_to("main.need_file", &locale));
                std::process::exit(1);
            }
        }
    }
}

/// 从命令参数中提取文件路径
fn get_file_from_command(cmd: &Command, cli_file: Option<&PathBuf>) -> PathBuf {
    // 优先使用全局 --file
    if let Some(f) = cli_file {
        if f.to_str() != Some("-") {
            return f.clone();
        }
    }

    // 从命令中获取
    match cmd {
        Command::Get { file, .. }
        | Command::Keys { file, .. }
        | Command::Len { file, .. }
        | Command::Type { file, .. }
        | Command::Exists { file, .. }
        | Command::Schema { file }
        | Command::Check { file }
        | Command::Set { file, .. }
        | Command::Del { file, .. }
        | Command::Add { file, .. }
        | Command::Patch { file, .. }
        | Command::Mv { file, .. }
        | Command::Fmt { file, .. }
        | Command::Fix { file, .. }
        | Command::Minify { file }
        | Command::Diff { file, .. }
        | Command::Tree { file, .. }
        | Command::Query { file, .. }
        | Command::Convert { file, .. } => file.clone(),

        Command::Validate { file, .. } => file.clone(),

        _ => PathBuf::from("-"),
    }
}

/// 解析输入文件（支持 stdin）
fn resolve_input_file(file: Option<&PathBuf>) -> PathBuf {
    let Some(f) = file else {
        eprintln!("Error: No input file specified");
        std::process::exit(1);
    };

    let f_str = f.to_str().unwrap_or("-");
    if f_str == "-" {
        // 从 stdin 读取
        let mut input = String::new();
        if std::io::stdin().read_to_string(&mut input).is_err() {
            eprintln!("Error: Failed to read from stdin");
            std::process::exit(1);
        }
        let mut temp_path = std::env::temp_dir();
        temp_path.push("jed_stdin.json");
        if let Err(e) = std::fs::write(&temp_path, &input) {
            eprintln!("Error: Failed to write temp file: {}", e);
            std::process::exit(1);
        }
        return temp_path;
    }

    f.clone()
}

/// 打印所有可用命令
fn print_commands(json_output: bool) {
    let commands = serde_json::json!([
        {"name": "get", "description": "Get value at path"},
        {"name": "keys", "description": "List all keys or indices"},
        {"name": "len", "description": "Get array length or key count"},
        {"name": "type", "description": "Get value type"},
        {"name": "exists", "description": "Check if path exists"},
        {"name": "schema", "description": "Generate structure summary"},
        {"name": "check", "description": "Validate JSON format"},
        {"name": "set", "description": "Set value at path"},
        {"name": "del", "description": "Delete key or element"},
        {"name": "add", "description": "Append to array or merge to object"},
        {"name": "patch", "description": "Batch operations (JSON Patch)"},
        {"name": "mv", "description": "Move/rename key"},
        {"name": "fmt", "description": "Format JSON"},
        {"name": "fix", "description": "Auto-fix JSON errors"},
        {"name": "minify", "description": "Minify JSON"},
        {"name": "diff", "description": "Compare two JSON files"},
        {"name": "tree", "description": "Show tree structure (non-interactive)"},
        {"name": "query", "description": "Filter/query JSON (jq-like)"},
        {"name": "validate", "description": "Validate against JSON Schema"},
        {"name": "convert", "description": "Convert to other formats"},
        {"name": "commands", "description": "List all commands"},
        {"name": "explain", "description": "Get command help"},
        {"name": "completions", "description": "Generate shell completions"}
    ]);

    if json_output {
        println!("{{\"ok\":true,\"commands\":{}}}", commands);
    } else {
        println!("Available commands:");
        for cmd in commands.as_array().unwrap() {
            println!("  {}  - {}", cmd["name"], cmd["description"]);
        }
    }
}

/// 打印命令帮助
fn print_command_help(cmd_name: &str, json_output: bool) {
    let help = match cmd_name {
        "get" => serde_json::json!({
            "name": "get",
            "usage": "jed [FILE] get <PATH>",
            "description": "Get value at path (Agent-friendly)",
            "example": "jed config.json get .database.host"
        }),
        "set" => serde_json::json!({
            "name": "set",
            "usage": "jed [FILE] set <PATH> <VALUE>",
            "description": "Set value at path",
            "example": "jed config.json set .debug true"
        }),
        _ => serde_json::json!({"error": format!("Unknown command: {}", cmd_name)}),
    };

    if json_output {
        println!("{{\"ok\":true,\"help\":{}}}", help);
    } else {
        println!("Command: {}", help["name"]);
        println!("Usage: {}", help["usage"]);
        println!("Description: {}", help["description"]);
        println!("Example: {}", help["example"]);
    }
}
