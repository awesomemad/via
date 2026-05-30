use anyhow::anyhow;
use std::ffi::{CStr, CString};
use std::sync::Mutex;
use via_engine::OmniseyeEngine;

// Global engine instance, lazily initialized, thread-safe.
static ENGINE: Mutex<Option<OmniseyeEngine>> = Mutex::new(None);

fn with_engine<F, R>(f: F) -> Result<R, String>
where
    F: FnOnce(&OmniseyeEngine) -> Result<R, anyhow::Error>,
{
    let guard = ENGINE.lock().map_err(|e| e.to_string())?;
    let engine = guard.as_ref().ok_or_else(|| "Engine not initialized. Call via_create() first.".to_string())?;
    f(engine).map_err(|e| e.to_string())
}

#[no_mangle]
pub extern "C" fn via_create() -> i32 {
    let mut guard = match ENGINE.lock() {
        Ok(g) => g,
        Err(_) => return -1,
    };
    if guard.is_some() {
        return 0;
    }
    match OmniseyeEngine::new() {
        Ok(e) => {
            *guard = Some(e);
            0
        }
        Err(err) => {
            let _ = err;
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn via_destroy() {
    if let Ok(mut guard) = ENGINE.lock() {
        *guard = None;
    }
}

fn cmd_result_json(success: bool, value: serde_json::Value) -> CString {
    let obj = serde_json::json!({
        "success": success,
        "data": value,
    });
    CString::new(obj.to_string()).unwrap_or_default()
}

#[no_mangle]
pub extern "C" fn via_dump() -> *mut std::ffi::c_char {
    let result = with_engine(|e| e.dump_ui_tree());
    let c_str = match result {
        Ok(dump) => CString::new(dump).unwrap_or_default(),
        Err(e) => CString::new(format!("Error: {}", e)).unwrap_or_default(),
    };
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn via_enum_windows() -> *mut std::ffi::c_char {
    let c_str = match with_engine(|e| e.enum_windows()) {
        Ok(windows) => {
            let json = serde_json::to_string(&windows).unwrap_or_default();
            CString::new(json).unwrap_or_default()
        }
        Err(e) => CString::new(format!("Error: {}", e)).unwrap_or_default(),
    };
    c_str.into_raw()
}

#[no_mangle]
pub extern "C" fn via_command(action: *const std::ffi::c_char, params_json: *const std::ffi::c_char) -> *mut std::ffi::c_char {
    let action = match unsafe { CStr::from_ptr(action) }.to_str() {
        Ok(s) => s,
        Err(_) => return cmd_result_json(false, serde_json::json!("Invalid action string")).into_raw(),
    };

    let params: serde_json::Value = if params_json.is_null() {
        serde_json::Value::Object(Default::default())
    } else {
        let s = match unsafe { CStr::from_ptr(params_json) }.to_str() {
            Ok(s) => s,
            Err(_) => return cmd_result_json(false, serde_json::json!("Invalid params string")).into_raw(),
        };
        serde_json::from_str(s).unwrap_or(serde_json::Value::Object(Default::default()))
    };

    let result = with_engine(|engine| execute_command(engine, action, &params));

    let out = match result {
        Ok(val) => cmd_result_json(true, val),
        Err(e) => cmd_result_json(false, serde_json::json!(e)),
    };
    out.into_raw()
}

fn execute_command(engine: &OmniseyeEngine, action: &str, params: &serde_json::Value) -> anyhow::Result<serde_json::Value> {
    match action {
        "click" => {
            let x = get_param_i32(params, "x")?;
            let y = get_param_i32(params, "y")?;
            let button = params.get("button").and_then(|v| v.as_str()).unwrap_or("left");
            let btn = match button {
                "right" => via_engine::input::MouseButton::Right,
                "middle" => via_engine::input::MouseButton::Middle,
                _ => via_engine::input::MouseButton::Left,
            };
            engine.click(x, y, btn)?;
            Ok(serde_json::json!({"x": x, "y": y, "button": button}))
        }
        "double_click" => {
            let x = get_param_i32(params, "x")?;
            let y = get_param_i32(params, "y")?;
            engine.double_click(x, y)?;
            Ok(serde_json::json!({"x": x, "y": y}))
        }
        "type" => {
            let text = get_param_str(params, "text")?;
            engine.type_text(text)?;
            Ok(serde_json::json!({"text": text}))
        }
        "move_mouse" => {
            let x = get_param_i32(params, "x")?;
            let y = get_param_i32(params, "y")?;
            engine.move_mouse(x, y)?;
            Ok(serde_json::json!({"x": x, "y": y}))
        }
        "focus" => {
            let hwnd = get_param_hwnd(params)?;
            engine.focus_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "minimize" => {
            let hwnd = get_param_hwnd(params)?;
            engine.minimize_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "maximize" => {
            let hwnd = get_param_hwnd(params)?;
            engine.maximize_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "restore" => {
            let hwnd = get_param_hwnd(params)?;
            engine.restore_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "close" => {
            let hwnd = get_param_hwnd(params)?;
            engine.close_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "hide" => {
            let hwnd = get_param_hwnd(params)?;
            engine.hide_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "show" => {
            let hwnd = get_param_hwnd(params)?;
            engine.show_window(hwnd)?;
            Ok(serde_json::json!({"hwnd": hwnd}))
        }
        "move_window" => {
            let hwnd = get_param_hwnd(params)?;
            let x = get_param_i32(params, "x")?;
            let y = get_param_i32(params, "y")?;
            let w = get_param_i32(params, "w")?;
            let h = get_param_i32(params, "h")?;
            engine.move_window(hwnd, x, y, w, h)?;
            Ok(serde_json::json!({"hwnd": hwnd, "x": x, "y": y, "w": w, "h": h}))
        }
        "resize" => {
            let hwnd = get_param_hwnd(params)?;
            let w = get_param_i32(params, "w")?;
            let h = get_param_i32(params, "h")?;
            engine.resize_window(hwnd, w, h)?;
            Ok(serde_json::json!({"hwnd": hwnd, "w": w, "h": h}))
        }
        "enum_windows" => {
            let windows = engine.enum_windows()?;
            Ok(serde_json::to_value(windows).unwrap_or_default())
        }
        "get_window_info" => {
            let hwnd = get_param_hwnd(params)?;
            let info = engine.get_window_info(hwnd)?;
            Ok(serde_json::to_value(info).unwrap_or_default())
        }
        "switch_desktop" => {
            let direction = get_param_i32(params, "direction").unwrap_or(1);
            engine.switch_desktop(direction)?;
            Ok(serde_json::json!({"direction": direction}))
        }
        "key_press" => {
            let vk = get_param_u16(params, "vk")?;
            engine.key_press(vk)?;
            Ok(serde_json::json!({"vk": vk}))
        }
        "key_release" => {
            let vk = get_param_u16(params, "vk")?;
            engine.key_release(vk)?;
            Ok(serde_json::json!({"vk": vk}))
        }
        "post_message" => {
            let hwnd = get_param_hwnd(params)?;
            let msg = get_param_u32(params, "msg")?;
            let wparam = get_param_usize(params, "wparam").unwrap_or(0);
            let lparam = get_param_isize(params, "lparam").unwrap_or(0);
            engine.post_message(hwnd, msg, wparam, lparam)?;
            Ok(serde_json::json!({"hwnd": hwnd, "msg": msg}))
        }
        "dump" => {
            let dump = engine.dump_ui_tree()?;
            Ok(serde_json::json!({"dump": dump}))
        }
        _ => Err(anyhow!("Unknown action: {}", action)),
    }
}

fn get_param_i32(params: &serde_json::Value, key: &str) -> anyhow::Result<i32> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as i32).ok_or_else(|| anyhow!("Missing or invalid i32 param: {}", key))
}

fn get_param_u16(params: &serde_json::Value, key: &str) -> anyhow::Result<u16> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as u16).ok_or_else(|| anyhow!("Missing or invalid u16 param: {}", key))
}

fn get_param_u32(params: &serde_json::Value, key: &str) -> anyhow::Result<u32> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as u32).ok_or_else(|| anyhow!("Missing or invalid u32 param: {}", key))
}

fn get_param_usize(params: &serde_json::Value, key: &str) -> Option<usize> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as usize)
}

fn get_param_isize(params: &serde_json::Value, key: &str) -> Option<isize> {
    params.get(key).and_then(|v| v.as_i64()).map(|v| v as isize)
}

fn get_param_hwnd(params: &serde_json::Value) -> anyhow::Result<isize> {
    params.get("hwnd")
        .and_then(|v| {
            v.as_i64().map(|n| n as isize)
                .or_else(|| v.as_str().and_then(|s| isize::from_str_radix(s.trim_start_matches("0x"), 16).ok()))
        })
        .ok_or_else(|| anyhow!("Missing or invalid param: hwnd"))
}

fn get_param_str<'a>(params: &'a serde_json::Value, key: &str) -> anyhow::Result<&'a str> {
    params.get(key).and_then(|v| v.as_str()).ok_or_else(|| anyhow!("Missing or invalid string param: {}", key))
}

#[no_mangle]
pub extern "C" fn via_free_string(s: *mut std::ffi::c_char) {
    if !s.is_null() {
        unsafe { let _ = CString::from_raw(s); }
    }
}
