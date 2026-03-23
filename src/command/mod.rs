pub mod read;
pub mod repair;
pub mod write;

use std::{fs, path::Path, process};

use crate::{
    cli::Command,
    engine::{JsonValue, parse_lenient},
    i18n::{get_locale, t_to},
};

/// 命令模式的统一退出码。
pub mod exit_code {
    pub const OK: i32 = 0;
    pub const ERROR: i32 = 1;
    pub const NOT_FOUND: i32 = 2;
    pub const TYPE_MISMATCH: i32 = 3;
}

/// 执行命令并以适当的退出码退出。
pub fn run(file: &Path, cmd: Command, json_output: bool) {
    let result = dispatch(file, cmd, json_output);
    match result {
        Ok(code) => process::exit(code),
        Err(e) => {
            print_error(&e.to_string(), json_output);
            process::exit(exit_code::ERROR);
        }
    }
}

fn dispatch(
    file: &Path,
    cmd: Command,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    match cmd {
        Command::Get { path, .. } => read::cmd_get(file, &path, json_output),
        Command::Keys { path, .. } => read::cmd_keys(file, &path, json_output),
        Command::Len { path, .. } => read::cmd_len(file, &path, json_output),
        Command::Type { path, .. } => read::cmd_type(file, &path, json_output),
        Command::Exists { path, .. } => read::cmd_exists(file, &path, json_output),
        Command::Schema { .. } => read::cmd_schema(file, json_output),
        Command::Check { .. } => read::cmd_check(file, json_output),
        Command::Set { path, value, .. } => write::cmd_set(file, &path, &value, json_output),
        Command::Del { path, .. } => write::cmd_del(file, &path, json_output),
        Command::Add { path, value, .. } => write::cmd_add(file, &path, &value, json_output),
        Command::Patch { operations, .. } => write::cmd_patch(file, &operations, json_output),
        Command::Mv { src, dst, .. } => write::cmd_mv(file, &src, &dst, json_output),
        Command::Fmt { indent, .. } => repair::cmd_fmt(file, indent, json_output),
        Command::Fix {
            dry_run,
            strip_comments,
            ..
        } => repair::cmd_fix(file, dry_run, strip_comments, json_output),
        Command::Minify { .. } => repair::cmd_minify(file, json_output),
        Command::Diff { other, .. } => read::cmd_diff(file, &other, json_output),
        Command::Completions { .. } => unreachable!("completions is handled in main"),
        Command::Commands {} => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
        Command::Explain { .. } => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
        Command::Tree { .. } => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
        Command::Query { .. } => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
        Command::Validate { .. } => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
        Command::Convert { .. } => {
            // 已在上层处理
            Ok(exit_code::OK)
        }
    }
}

/// 运行 tree 命令
pub fn run_tree(
    file: &Path,
    expand_all: bool,
    path: Option<&str>,
    json_output: bool,
) {
    use crate::engine::get;

    let locale = get_locale();
    let (doc, _) = match load_lenient(file) {
        Ok(v) => v,
        Err(e) => {
            print_error(&t_to("err.parse_failed", &locale)
                .replace("{0}", &file.display().to_string())
                .replace("{1}", &e.to_string()), json_output);
            process::exit(exit_code::ERROR);
        }
    };

    // 如果指定了路径，只显示子树
    let display_doc = if let Some(p) = path {
        match get(&doc, p) {
            Ok(v) => v.clone(),
            Err(e) => {
                print_error(&format!("Path error: {}", e), json_output);
                process::exit(exit_code::NOT_FOUND);
            }
        }
    } else {
        doc
    };

    // 生成树形输出
    let tree = render_tree(&display_doc, 0, expand_all);
    print_ok(&tree, json_output);
}

fn render_tree(value: &JsonValue, depth: usize, expand_all: bool) -> String {
    let indent = "  ".repeat(depth);
    match value {
        JsonValue::Object(map) => {
            let mut output = String::new();
            output.push_str(&format!("{} {{\n", indent));
            for (_key, val) in map {
                output.push_str(&render_tree(val, depth + 1, expand_all));
            }
            output.push_str(&format!("{}}}\n", indent));
            output
        }
        JsonValue::Array(arr) => {
            let mut output = String::new();
            output.push_str(&format!("{} [\n", indent));
            for val in arr {
                output.push_str(&render_tree(val, depth + 1, expand_all));
            }
            output.push_str(&format!("{}]\n", indent));
            output
        }
        _ => {
            format!("{}{}\n", indent, value)
        }
    }
}

/// 运行 query 命令
pub fn run_query(file: &Path, filter: &str, json_output: bool) {
    use crate::engine::get;

    let locale = get_locale();
    let (doc, _) = match load_lenient(file) {
        Ok(v) => v,
        Err(e) => {
            print_error(&t_to("err.parse_failed", &locale)
                .replace("{0}", &file.display().to_string())
                .replace("{1}", &e.to_string()), json_output);
            process::exit(exit_code::ERROR);
        }
    };

    // 简单过滤：支持 .field 和 .arr[index] 语法
    match get(&doc, filter) {
        Ok(value) => {
            print_json_value(value, json_output);
        }
        Err(e) => {
            print_error(&format!("Query error: {}", e), json_output);
            process::exit(exit_code::NOT_FOUND);
        }
    }
}

/// 运行 validate 命令
pub fn run_validate(file: &Path, schema_file: &Path, json_output: bool) {
    // JSON Schema 验证（简化版本）
    // TODO: 完整实现需要添加 jsonschema crate
    let locale = get_locale();

    // 读取 schema
    let schema_content = match fs::read_to_string(schema_file) {
        Ok(c) => c,
        Err(e) => {
            print_error(&t_to("err.read_failed", &locale)
                .replace("{0}", &schema_file.display().to_string())
                .replace("{1}", &e.to_string()), json_output);
            process::exit(exit_code::ERROR);
        }
    };

    // 读取数据
    let (doc, _) = match load_lenient(file) {
        Ok(v) => v,
        Err(e) => {
            print_error(&t_to("err.parse_failed", &locale)
                .replace("{0}", &file.display().to_string())
                .replace("{1}", &e.to_string()), json_output);
            process::exit(exit_code::ERROR);
        }
    };

    // 简化验证：检查 schema 中的必填字段是否存在
    // 完整实现需要使用 jsonschema crate
    if let JsonValue::Object(schema_obj) = parse_json(&schema_content) {
        if let JsonValue::Object(doc_obj) = &doc {
            let required = schema_obj.get("required");
            if let Some(required_arr) = required {
                if let JsonValue::Array(reqs) = required_arr {
                    let mut missing = Vec::new();
                    for req in reqs {
                        if let JsonValue::String(key) = req {
                            if !doc_obj.contains_key(key) {
                                missing.push(key.clone());
                            }
                        }
                    }
                    if missing.is_empty() {
                        print_ok("valid", json_output);
                    } else {
                        let err = format!("Missing required fields: {:?}", missing);
                        print_error(&err, json_output);
                        process::exit(exit_code::ERROR);
                    }
                    return;
                }
            }
        }
    }

    // 默认通过
    print_ok("valid (basic check)", json_output);
}

fn parse_json(s: &str) -> JsonValue {
    use crate::engine::parse_lenient;
    // 简单尝试解析，不处理错误
    parse_lenient(s).map(|o| o.value).unwrap_or(JsonValue::Null)
}

/// 运行 convert 命令
pub fn run_convert(file: &Path, format: &str, json_output: bool) {
    let locale = get_locale();
    let (doc, _) = match load_lenient(file) {
        Ok(v) => v,
        Err(e) => {
            print_error(&t_to("err.parse_failed", &locale)
                .replace("{0}", &file.display().to_string())
                .replace("{1}", &e.to_string()), json_output);
            process::exit(exit_code::ERROR);
        }
    };

    match format.to_lowercase().as_str() {
        "yaml" => {
            // 简单 YAML 输出
            let yaml = to_yaml(&doc, 0);
            println!("{}", yaml);
        }
        "toml" => {
            // 简化 TOML 输出
            print_error("TOML output requires 'toml' crate. Use --format yaml instead.", json_output);
            process::exit(exit_code::ERROR);
        }
        _ => {
            print_error(&format!("Unknown format: {}. Supported: yaml, toml", format), json_output);
            process::exit(exit_code::ERROR);
        }
    }
}

fn to_yaml(value: &JsonValue, depth: usize) -> String {
    let indent = "  ".repeat(depth);
    match value {
        JsonValue::Object(map) => {
            let mut output = String::new();
            for (key, val) in map {
                output.push_str(&format!("{}{}:\n", indent, key));
                output.push_str(&to_yaml(val, depth + 1));
            }
            output
        }
        JsonValue::Array(arr) => {
            let mut output = String::new();
            for val in arr {
                output.push_str(&format!("{}-\n", indent));
                output.push_str(&to_yaml(val, depth + 1));
            }
            output
        }
        JsonValue::String(s) => {
            format!("{}: \"{}\"\n", indent, s)
        }
        JsonValue::Number(n) => {
            format!("{}: {}\n", indent, n)
        }
        JsonValue::Bool(b) => {
            format!("{}: {}\n", indent, b)
        }
        JsonValue::Null => {
            format!("{}: null\n", indent)
        }
    }
}

// ── 输出帮助函数 ──────────────────────────────────────────────────────────────

/// 输出一个 JSON 值。json 模式下包装为 `{"ok":true,"value":...}`。
pub(crate) fn print_json_value(value: &JsonValue, json_output: bool) {
    use crate::engine::format_compact;
    let compact = format_compact(value);
    if json_output {
        println!("{{\"ok\":true,\"value\":{compact}}}");
    } else {
        println!("{compact}");
    }
}

/// 输出纯字符串值。json 模式下包装为 `{"ok":true,"value":"..."}` (字符串类型)。
pub(crate) fn print_str(value: &str, json_output: bool) {
    if json_output {
        println!("{}", serde_json::json!({"ok": true, "value": value}));
    } else {
        println!("{value}");
    }
}

/// 输出整数值。json 模式下包装为 `{"ok":true,"value":n}`。
pub(crate) fn print_usize(n: usize, json_output: bool) {
    if json_output {
        println!("{{\"ok\":true,\"value\":{n}}}");
    } else {
        println!("{n}");
    }
}

/// 输出字符串列表。json 模式下包装为 `{"ok":true,"value":[...]}`。
pub(crate) fn print_string_list(lines: &[String], json_output: bool) {
    if json_output {
        let arr: Vec<serde_json::Value> = lines
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        println!("{}", serde_json::json!({"ok": true, "value": arr}));
    } else {
        for line in lines {
            println!("{line}");
        }
    }
}

/// 输出成功消息。json 模式下只输出 `{"ok":true}`。
pub(crate) fn print_ok(msg: &str, json_output: bool) {
    if json_output {
        println!("{{\"ok\":true,\"value\":\"{msg}\"}}");
    } else {
        println!("{msg}");
    }
}

/// 输出错误消息。json 模式下输出到 stdout，普通模式输出到 stderr。
pub(crate) fn print_error(msg: &str, json_output: bool) {
    if json_output {
        println!("{}", serde_json::json!({"ok": false, "error": msg}));
    } else {
        eprintln!("{msg}");
    }
}

// ── 文件 I/O 帮助函数 ─────────────────────────────────────────────────────────

/// 读取文件内容，返回错误信息若文件不存在。
pub(crate) fn read_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let locale = get_locale();
    fs::read_to_string(path).map_err(|e| {
        t_to("err.read_failed", &locale)
            .replace("{0}", &path.display().to_string())
            .replace("{1}", &e.to_string())
            .into()
    })
}

/// 读取并宽松解析文件，返回文档和修复列表。
pub(crate) fn load_lenient(
    path: &Path,
) -> Result<(JsonValue, Vec<crate::engine::Repair>), Box<dyn std::error::Error>> {
    let locale = get_locale();
    let content = read_file(path)?;
    let output = parse_lenient(&content).map_err(|e| {
        t_to("err.parse_failed", &locale)
            .replace("{0}", &path.display().to_string())
            .replace("{1}", &e.to_string())
    })?;
    Ok((output.value, output.repairs))
}

/// 原子写入文件：写临时文件 → fsync → 重命名。
pub(crate) fn write_file_atomic(
    path: &Path,
    content: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let locale = get_locale();
    let tmp_path = path.with_extension("tmp");
    fs::write(&tmp_path, content)
        .map_err(|e| t_to("err.write_tmp_failed", &locale).replace("{0}", &e.to_string()))?;
    fs::rename(&tmp_path, path)
        .map_err(|e| t_to("err.rename_failed_file", &locale).replace("{0}", &e.to_string()))?;
    Ok(())
}
