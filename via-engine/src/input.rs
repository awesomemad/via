use anyhow::{Context, Result};
use serde::Serialize;
use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, PostMessageW, SYSTEM_METRICS_INDEX};

#[derive(Debug, Clone, Copy, Serialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

fn vk_from_char(c: char) -> Option<VIRTUAL_KEY> {
    match c {
        'a'..='z' => Some(VIRTUAL_KEY(c as u16 - 0x20)),
        'A'..='Z' => Some(VIRTUAL_KEY(c as u16)),
        '0'..='9' => Some(VIRTUAL_KEY(c as u16)),
        ' ' => Some(VIRTUAL_KEY(0x20)),
        '\t' => Some(VIRTUAL_KEY(0x09)),
        '\n' | '\r' => Some(VIRTUAL_KEY(0x0D)),
        '.' => Some(VIRTUAL_KEY(0xBE)),
        ',' => Some(VIRTUAL_KEY(0xBC)),
        '!' => Some(VIRTUAL_KEY(0x31)),
        '@' => Some(VIRTUAL_KEY(0x32)),
        '#' => Some(VIRTUAL_KEY(0x33)),
        '$' => Some(VIRTUAL_KEY(0x34)),
        '%' => Some(VIRTUAL_KEY(0x35)),
        '^' => Some(VIRTUAL_KEY(0x36)),
        '&' => Some(VIRTUAL_KEY(0x37)),
        '*' => Some(VIRTUAL_KEY(0x38)),
        '(' => Some(VIRTUAL_KEY(0x39)),
        ')' => Some(VIRTUAL_KEY(0x30)),
        '-' => Some(VIRTUAL_KEY(0xBD)),
        '_' => Some(VIRTUAL_KEY(0xBD)),
        '=' => Some(VIRTUAL_KEY(0xBB)),
        '+' => Some(VIRTUAL_KEY(0xBB)),
        '[' => Some(VIRTUAL_KEY(0xDB)),
        ']' => Some(VIRTUAL_KEY(0xDD)),
        '{' => Some(VIRTUAL_KEY(0xDB)),
        '}' => Some(VIRTUAL_KEY(0xDD)),
        '\\' => Some(VIRTUAL_KEY(0xDC)),
        '|' => Some(VIRTUAL_KEY(0xDC)),
        ';' => Some(VIRTUAL_KEY(0xBA)),
        ':' => Some(VIRTUAL_KEY(0xBA)),
        '\'' => Some(VIRTUAL_KEY(0xDE)),
        '"' => Some(VIRTUAL_KEY(0xDE)),
        '/' => Some(VIRTUAL_KEY(0xBF)),
        '?' => Some(VIRTUAL_KEY(0xBF)),
        '`' => Some(VIRTUAL_KEY(0xC0)),
        '~' => Some(VIRTUAL_KEY(0xC0)),
        '<' => Some(VIRTUAL_KEY(0xBC)),
        '>' => Some(VIRTUAL_KEY(0xBE)),
        _ => None,
    }
}

fn needs_shift(c: char) -> bool {
    matches!(
        c,
        'A'..='Z' | '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '_' | '+' | '{' | '}' | '|' | ':' | '"' | '?' | '>' | '~'
    )
}

fn make_mouse_input(dw_flags: MOUSE_EVENT_FLAGS, dx: i32, dy: i32) -> INPUT {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_MOUSE;
    input.Anonymous.mi.dx = dx;
    input.Anonymous.mi.dy = dy;
    input.Anonymous.mi.mouseData = 0;
    input.Anonymous.mi.dwFlags = dw_flags;
    input
}

fn make_key_input(w_vk: VIRTUAL_KEY, dw_flags: KEYBD_EVENT_FLAGS) -> INPUT {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki.wVk = w_vk;
    input.Anonymous.ki.wScan = 0;
    input.Anonymous.ki.dwFlags = dw_flags;
    input.Anonymous.ki.time = 0;
    input.Anonymous.ki.dwExtraInfo = 0;
    input
}

fn send(inputs: &[INPUT]) {
    unsafe { SendInput(inputs, std::mem::size_of::<INPUT>() as i32); }
}

pub fn click(x: i32, y: i32, button: MouseButton) -> Result<()> {
    move_mouse(x, y)?;

    let (down, up) = match button {
        MouseButton::Left => (MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP),
        MouseButton::Right => (MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP),
        MouseButton::Middle => (MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP),
    };

    send(&[make_mouse_input(down, 0, 0), make_mouse_input(up, 0, 0)]);
    Ok(())
}

pub fn double_click(x: i32, y: i32) -> Result<()> {
    click(x, y, MouseButton::Left)?;
    std::thread::sleep(std::time::Duration::from_millis(30));
    click(x, y, MouseButton::Left)?;
    Ok(())
}

pub fn move_mouse(x: i32, y: i32) -> Result<()> {
    let sw = unsafe { GetSystemMetrics(SYSTEM_METRICS_INDEX(0)) };
    let sh = unsafe { GetSystemMetrics(SYSTEM_METRICS_INDEX(1)) };

    let abs_x = ((x as i64) * 65535 / (sw.max(1) - 1) as i64) as i32;
    let abs_y = ((y as i64) * 65535 / (sh.max(1) - 1) as i64) as i32;

    send(&[make_mouse_input(MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE, abs_x, abs_y)]);
    Ok(())
}

pub fn type_text(text: &str) -> Result<()> {
    for ch in text.chars() {
        if ch == '\r' { continue; }
        if ch == '\n' {
            send(&[make_key_input(VIRTUAL_KEY(0x0D), KEYBD_EVENT_FLAGS(0)), make_key_input(VIRTUAL_KEY(0x0D), KEYEVENTF_KEYUP)]);
            continue;
        }

        let vk = vk_from_char(ch).context(format!("Unsupported char: {:?}", ch))?;
        let shift = needs_shift(ch);

        if shift {
            send(&[make_key_input(VIRTUAL_KEY(0x10), KEYBD_EVENT_FLAGS(0))]);
        }

        send(&[
            make_key_input(vk, KEYBD_EVENT_FLAGS(0)),
            make_key_input(vk, KEYEVENTF_KEYUP),
        ]);

        if shift {
            send(&[make_key_input(VIRTUAL_KEY(0x10), KEYEVENTF_KEYUP)]);
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    Ok(())
}

pub fn key_press(vk: u16) -> Result<()> {
    send(&[make_key_input(VIRTUAL_KEY(vk), KEYBD_EVENT_FLAGS(0))]);
    Ok(())
}

pub fn key_release(vk: u16) -> Result<()> {
    send(&[make_key_input(VIRTUAL_KEY(vk), KEYEVENTF_KEYUP)]);
    Ok(())
}

pub fn post_message(hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> Result<()> {
    let handle = HWND(hwnd as *mut _);
    unsafe {
        PostMessageW(handle, msg, WPARAM(wparam), LPARAM(lparam))
            .ok()
            .context("PostMessageW failed")?;
    }
    Ok(())
}
