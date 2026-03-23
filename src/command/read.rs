use std::path::Path;

use crate::{
    command::{
        exit_code, load_lenient, print_error, print_json_value, print_ok, print_str,
        print_string_list, print_usize, read_file,
    },
    engine::{FormatOptions, JsonValue, PathError, exists, format_pretty, get, infer_schema, parse_lenient},
    i18n::{get_locale, t_to},
};

/// `get <path>` — 输出路径处的值，Agent 友好（最小化输出）。
pub fn cmd_get(
    file: &Path,
    path: &str,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let locale = get_locale();
    let (doc, _) = load_lenient(file)?;
    match get(&doc, path) {
        Ok(value) => {
            print_json_value(value, json_output);
            Ok(exit_code::OK)
        }
        Err(PathError::KeyNotFound { key }) => {
            print_error(
                &t_to("err.key_not_found", &locale).replace("{0}", &key),
                json_output,
            );
            Ok(exit_code::NOT_FOUND)
        }
        Err(PathError::IndexOutOfBounds { index, len }) => {
            print_error(
                &t_to("err.index_oob", &locale)
                    .replace("{0}", &index.to_string())
                    .replace("{1}", &len.to_string()),
                json_output,
            );
            Ok(exit_code::NOT_FOUND)
        }
        Err(e) => {
            print_error(
                &t_to("err.path", &locale).replace("{0}", &e.to_string()),
                json_output,
            );
            Ok(exit_code::TYPE_MISMATCH)
        }
    }
}

/// `keys <path>` — 每行输出一个 key 或索引。
pub fn cmd_keys(
    file: &Path,
    path: &str,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let locale = get_locale();
    let (doc, _) = load_lenient(file)?;
    let node = match get(&doc, path) {
        Ok(v) => v,
        Err(e) => {
            print_error(
                &t_to("err.path", &locale).replace("{0}", &e.to_string()),
                json_output,
            );
            return Ok(exit_code::NOT_FOUND);
        }
    };

    match node {
        crate::engine::JsonValue::Object(map) => {
            let keys: Vec<String> = map.keys().cloned().collect();
            print_string_list(&keys, json_output);
        }
        crate::engine::JsonValue::Array(arr) => {
            let indices: Vec<String> = (0..arr.len()).map(|i| i.to_string()).collect();
            print_string_list(&indices, json_output);
        }
        other => {
            print_error(
                &t_to("err.type_no_keys", &locale).replace("{0}", other.type_name()),
                json_output,
            );
            return Ok(exit_code::TYPE_MISMATCH);
        }
    }
    Ok(exit_code::OK)
}

/// `len <path>` — 输出数组长度或对象 key 数量。
pub fn cmd_len(
    file: &Path,
    path: &str,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let locale = get_locale();
    let (doc, _) = load_lenient(file)?;
    let node = match get(&doc, path) {
        Ok(v) => v,
        Err(e) => {
            print_error(
                &t_to("err.path", &locale).replace("{0}", &e.to_string()),
                json_output,
            );
            return Ok(exit_code::NOT_FOUND);
        }
    };

    if let Some(n) = node.len() {
        print_usize(n, json_output);
        Ok(exit_code::OK)
    } else {
        print_error(
            &t_to("err.type_no_len", &locale).replace("{0}", node.type_name()),
            json_output,
        );
        Ok(exit_code::TYPE_MISMATCH)
    }
}

/// `type <path>` — 输出值的类型名称。
pub fn cmd_type(
    file: &Path,
    path: &str,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let locale = get_locale();
    let (doc, _) = load_lenient(file)?;
    match get(&doc, path) {
        Ok(v) => {
            print_str(v.type_name(), json_output);
            Ok(exit_code::OK)
        }
        Err(e) => {
            print_error(
                &t_to("err.path", &locale).replace("{0}", &e.to_string()),
                json_output,
            );
            Ok(exit_code::NOT_FOUND)
        }
    }
}

/// `exists <path>` — exit 0 表示存在，exit 2 表示不存在。
pub fn cmd_exists(
    file: &Path,
    path: &str,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let locale = get_locale();
    let (doc, _) = load_lenient(file)?;
    if exists(&doc, path) {
        if json_output {
            println!("{{\"ok\":true}}");
        }
        Ok(exit_code::OK)
    } else {
        if json_output {
            println!(
                "{}",
                serde_json::json!({"ok": false, "error": t_to("err.path_not_exists", &locale)})
            );
        }
        Ok(exit_code::NOT_FOUND)
    }
}

/// `schema` — 推断并输出文件结构（不含实际值）。
pub fn cmd_schema(file: &Path, json_output: bool) -> Result<i32, Box<dyn std::error::Error>> {
    let (doc, _) = load_lenient(file)?;

    // JSON 模式输出标准 JSON Schema 格式
    if json_output {
        let schema = build_json_schema(&doc);
        println!("{}", serde_json::to_string(&schema).unwrap_or_default());
    } else {
        // 人类友好的简洁格式
        print_str(&infer_schema(&doc), json_output);
    }
    Ok(exit_code::OK)
}

/// 构建标准 JSON Schema 格式
fn build_json_schema(value: &JsonValue) -> serde_json::Value {
    match value {
        JsonValue::Null => serde_json::json!({"type": "null"}),
        JsonValue::Bool(_) => serde_json::json!({"type": "boolean"}),
        JsonValue::Number(_) => serde_json::json!({"type": "number"}),
        JsonValue::String(_) => serde_json::json!({"type": "string"}),
        JsonValue::Array(arr) => {
            if arr.is_empty() {
                serde_json::json!({"type": "array", "items": {}})
            } else {
                // 取第一个元素的类型作为参考
                let items = if !arr.is_empty() {
                    build_json_schema(&arr[0])
                } else {
                    serde_json::json!({})
                };
                serde_json::json!({"type": "array", "items": items})
            }
        }
        JsonValue::Object(map) => {
            let mut props = serde_json::Map::new();
            for (k, v) in map {
                props.insert(k.clone(), build_json_schema(v));
            }
            serde_json::json!({
                "type": "object",
                "properties": props
            })
        }
    }
}

/// `check` — 校验 JSON，成功无输出，错误输出到 stderr。
pub fn cmd_check(file: &Path, json_output: bool) -> Result<i32, Box<dyn std::error::Error>> {
    let content = read_file(file)?;
    match parse_lenient(&content) {
        Ok(_) => {
            if json_output {
                println!("{{\"ok\":true}}");
            }
            Ok(exit_code::OK)
        }
        Err(e) => {
            print_error(&format!("{e}"), json_output);
            Ok(exit_code::ERROR)
        }
    }
}

/// `diff <other>` — 输出两个 JSON 文件的结构差异。
pub fn cmd_diff(
    file: &Path,
    other: &Path,
    json_output: bool,
) -> Result<i32, Box<dyn std::error::Error>> {
    let (a, _) = load_lenient(file)?;
    let (b, _) = load_lenient(other)?;

    let a_str = format_pretty(&a, &FormatOptions::default());
    let b_str = format_pretty(&b, &FormatOptions::default());

    if a_str == b_str {
        print_ok("identical", json_output);
        return Ok(exit_code::OK);
    }

    let a_lines: Vec<&str> = a_str.lines().collect();
    let b_lines: Vec<&str> = b_str.lines().collect();
    let max = a_lines.len().max(b_lines.len());

    let mut diff_lines: Vec<String> = Vec::new();
    for i in 0..max {
        match (a_lines.get(i), b_lines.get(i)) {
            (Some(al), Some(bl)) if al == bl => {}
            (Some(al), Some(bl)) => {
                diff_lines.push(format!("- {al}"));
                diff_lines.push(format!("+ {bl}"));
            }
            (Some(al), None) => diff_lines.push(format!("- {al}")),
            (None, Some(bl)) => diff_lines.push(format!("+ {bl}")),
            (None, None) => {}
        }
    }

    if json_output {
        let lines: Vec<serde_json::Value> = diff_lines
            .iter()
            .map(|s| serde_json::Value::String(s.clone()))
            .collect();
        println!("{}", serde_json::json!({"ok": false, "diff": lines}));
    } else {
        for line in &diff_lines {
            println!("{line}");
        }
    }

    Ok(exit_code::ERROR)
}
