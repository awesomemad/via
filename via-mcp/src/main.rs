use anyhow::{Context, Result};
use via_engine::input::MouseButton;
use via_engine::OmniseyeEngine;
use serde::{Deserialize, Serialize};
use std::io::{self, BufRead, Write};

// ── JSON-RPC types ──────────────────────────────────────────────

#[derive(Deserialize)]
struct JsonRpcRequest {
    #[serde(default)]
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: Option<serde_json::Value>,
}

#[derive(Serialize)]
struct JsonRpcResponse {
    jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<serde_json::Value>,
}

fn json_error(id: Option<serde_json::Value>, code: i32, msg: &str) -> String {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: None,
        error: Some(JsonRpcError {
            code,
            message: msg.to_string(),
            data: None,
        }),
    };
    serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string())
}

fn json_result(id: Option<serde_json::Value>, result: serde_json::Value) -> String {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0",
        id,
        result: Some(result),
        error: None,
    };
    serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string())
}

// ── Tool definitions ────────────────────────────────────────────

fn tool_definitions() -> serde_json::Value {
    serde_json::json!({
        "tools": [
            {
                "name": "gui_dump",
                "description": "Returns a compressed semantic map of the entire GUI state via UIA. ",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "gui_command",
                "description": "Universal GUI command handler. Supports click, type, move, focus, minimize, maximize, close, switch_desktop, key_press, key_release, post_message.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action: click|double_click|type|move_mouse|focus|minimize|maximize|restore|close|hide|show|move_window|resize|enum_windows|get_window_info|switch_desktop|key_press|key_release|post_message"
                        },
                        "params": {
                            "type": "object",
                            "description": "Parameters for the action. See action-specific format."
                        }
                    },
                    "required": ["action"]
                }
            }
        ]
    })
}

// ── Action handler ──────────────────────────────────────────────

fn handle_tool_call(engine: &OmniseyeEngine, name: &str, args: serde_json::Value) -> String {
    let id = serde_json::json!(null);

    let result = match name {
        "gui_dump" => handle_gui_dump(engine),
        "gui_command" => handle_gui_command(engine, &args),
        _ => Err(anyhow::anyhow!("Unknown tool: {}", name)),
    };

    match result {
        Ok(val) => {
            let resp = JsonRpcResponse {
                jsonrpc: "2.0",
                id: Some(id),
                result: Some(serde_json::json!({
                    "content": [{"type": "text", "text": val}]
                })),
                error: None,
            };
            serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string())
        }
        Err(e) => json_error(Some(id), -1, &format!("Error: {}", e)),
    }
}

fn handle_gui_dump(engine: &OmniseyeEngine) -> Result<String> {
    engine.dump_ui_tree()
}

fn handle_gui_command(engine: &OmniseyeEngine, args: &serde_json::Value) -> Result<String> {
    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'action' field"))?;

    let params = args.get("params").and_then(|v| v.as_object()).cloned().unwrap_or_default();

    match action {
        "click" => {
            let x = get_param_i32(&params, "x")?;
            let y = get_param_i32(&params, "y")?;
            let button = match params.get("button").and_then(|v| v.as_str()) {
                Some("right") => MouseButton::Right,
                Some("middle") => MouseButton::Middle,
                _ => MouseButton::Left,
            };
            engine.click(x, y, button)?;
            Ok(format!("click({},{}) done", x, y))
        }
        "double_click" => {
            let x = get_param_i32(&params, "x")?;
            let y = get_param_i32(&params, "y")?;
            engine.double_click(x, y)?;
            Ok(format!("double_click({},{}) done", x, y))
        }
        "type" => {
            let text = get_param_str(&params, "text")?;
            engine.type_text(text)?;
            Ok(format!("typed {:?}", text))
        }
        "move_mouse" => {
            let x = get_param_i32(&params, "x")?;
            let y = get_param_i32(&params, "y")?;
            engine.move_mouse(x, y)?;
            Ok(format!("move_mouse({},{}) done", x, y))
        }
        "focus" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.focus_window(hwnd)?;
            Ok(format!("focus(0x{:x}) done", hwnd))
        }
        "minimize" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.minimize_window(hwnd)?;
            Ok(format!("minimize(0x{:x}) done", hwnd))
        }
        "maximize" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.maximize_window(hwnd)?;
            Ok(format!("maximize(0x{:x}) done", hwnd))
        }
        "restore" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.restore_window(hwnd)?;
            Ok(format!("restore(0x{:x}) done", hwnd))
        }
        "close" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.close_window(hwnd)?;
            Ok(format!("close(0x{:x}) done", hwnd))
        }
        "hide" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.hide_window(hwnd)?;
            Ok(format!("hide(0x{:x}) done", hwnd))
        }
        "show" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            engine.show_window(hwnd)?;
            Ok(format!("show(0x{:x}) done", hwnd))
        }
        "move_window" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            let x = get_param_i32(&params, "x")?;
            let y = get_param_i32(&params, "y")?;
            let w = get_param_i32(&params, "w").unwrap_or(0);
            let h = get_param_i32(&params, "h").unwrap_or(0);
            if w > 0 && h > 0 {
                engine.move_window(hwnd, x, y, w, h)?;
            } else {
                let info = engine.get_window_info(hwnd)?;
                engine.move_window(hwnd, x, y, info.rect.2 - info.rect.0, info.rect.3 - info.rect.1)?;
            }
            Ok(format!("move_window(0x{:x}) done", hwnd))
        }
        "resize" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            let w = get_param_i32(&params, "w")?;
            let h = get_param_i32(&params, "h")?;
            engine.resize_window(hwnd, w, h)?;
            Ok(format!("resize(0x{:x}) done", hwnd))
        }
        "enum_windows" => {
            let windows = engine.enum_windows()?;
            let mut lines = Vec::new();
            for w in &windows {
                lines.push(format!(
                    "0x{:x} {:?} v={} [{},{},{},{}] pid={}",
                    w.hwnd, w.title, w.visible, w.rect.0, w.rect.1, w.rect.2, w.rect.3, w.pid
                ));
            }
            Ok(lines.join("\n"))
        }
        "get_window_info" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            let info = engine.get_window_info(hwnd)?;
            Ok(serde_json::to_string(&info)?)
        }
        "switch_desktop" => {
            let dir = get_param_i32(&params, "direction").unwrap_or(1);
            engine.switch_desktop(dir)?;
            Ok(format!("switch_desktop({}) done", dir))
        }
        "key_press" => {
            let vk = get_param_u16(&params, "vk")?;
            engine.key_press(vk)?;
            Ok(format!("key_press(0x{:x}) done", vk))
        }
        "key_release" => {
            let vk = get_param_u16(&params, "vk")?;
            engine.key_release(vk)?;
            Ok(format!("key_release(0x{:x}) done", vk))
        }
        "post_message" => {
            let hwnd = get_param_hwnd(&params, "hwnd")?;
            let msg = get_param_u32(&params, "msg")?;
            let wparam = get_param_usize(&params, "wparam").unwrap_or(0);
            let lparam = get_param_isize(&params, "lparam").unwrap_or(0);
            engine.post_message(hwnd, msg, wparam, lparam)?;
            Ok(format!("post_message(0x{:x}) done", hwnd))
        }
        _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
    }
}

// ── Parameter helpers ───────────────────────────────────────────

fn get_param_i32(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<i32> {
    map.get(key)
        .and_then(|v| v.as_i64())
        .map(|v| v as i32)
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid int param: {}", key))
}

fn get_param_u16(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<u16> {
    map.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u16)
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid u16 param: {}", key))
}

fn get_param_u32(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<u32> {
    map.get(key)
        .and_then(|v| v.as_u64())
        .map(|v| v as u32)
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid u32 param: {}", key))
}

fn get_param_usize(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<usize> {
    map.get(key).and_then(|v| v.as_u64()).map(|v| v as usize)
}

fn get_param_isize(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Option<isize> {
    map.get(key).and_then(|v| v.as_i64()).map(|v| v as isize)
}

fn get_param_str<'a>(map: &'a serde_json::Map<String, serde_json::Value>, key: &str) -> Result<&'a str> {
    map.get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing or invalid string param: {}", key))
}

fn get_param_hwnd(map: &serde_json::Map<String, serde_json::Value>, key: &str) -> Result<isize> {
    let val = map
        .get(key)
        .ok_or_else(|| anyhow::anyhow!("Missing param: {}", key))?;

    if let Some(s) = val.as_str() {
        if s.starts_with("0x") || s.starts_with("0X") {
            return isize::from_str_radix(&s[2..], 16).map_err(|e| anyhow::anyhow!("Invalid hwnd hex: {}", e));
        }
        return s.parse::<isize>().map_err(|e| anyhow::anyhow!("Invalid hwnd: {}", e));
    }
    if let Some(n) = val.as_u64() {
        return Ok(n as isize);
    }
    if let Some(n) = val.as_i64() {
        return Ok(n as isize);
    }

    Err(anyhow::anyhow!("Cannot parse hwnd from {:?}", val))
}

// ── Initialize response ─────────────────────────────────────────

fn handle_initialize(id: Option<serde_json::Value>) -> String {
    json_result(id, serde_json::json!({
        "protocolVersion": "2025-03-26",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "via-mcp",
            "version": "0.1.0"
        }
    }))
}

// ── Main loop ───────────────────────────────────────────────────

fn main() -> Result<()> {
    let engine = OmniseyeEngine::new().context("Failed to initialize Omniseye engine")?;
    let stdin = io::stdin();
    let reader = stdin.lock();

    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let err = json_error(None, -32700, &format!("Parse error: {}", e));
                let _ = writeln!(io::stdout(), "{}", err);
                continue;
            }
        };

        let response = match req.method.as_str() {
            "initialize" => handle_initialize(req.id),
            "notifications/initialized" | "notifications/cancelled" => String::new(),
            "tools/list" => json_result(req.id, tool_definitions()),
            "tools/call" => {
                let args = req.params.unwrap_or(serde_json::json!({}));
                let name = args.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let arguments = args.get("arguments").cloned().unwrap_or(serde_json::json!({}));
                handle_tool_call(&engine, name, arguments)
            }
            _ => json_error(
                req.id,
                -32601,
                &format!("Method not found: {}", req.method),
            ),
        };

        if !response.is_empty() {
            let mut stdout = io::stdout().lock();
            let _ = writeln!(stdout, "{}", response);
            let _ = stdout.flush();
        }
    }

    Ok(())
}
