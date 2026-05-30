use std::ffi::c_void;

use anyhow::{Context, Result};
use serde::Serialize;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, RECT, TRUE, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub hwnd: isize,
    pub title: String,
    pub class_name: String,
    pub rect: (i32, i32, i32, i32),
    pub visible: bool,
    pub pid: u32,
}

pub fn enum_windows() -> Result<Vec<WindowInfo>> {
    let mut windows: Vec<WindowInfo> = Vec::new();

    unsafe {
        EnumWindows(
            Some(enum_window_callback),
            LPARAM(&mut windows as *mut _ as isize),
        )
        .ok()
        .context("EnumWindows failed")?;
    }

    Ok(windows)
}

extern "system" fn enum_window_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = unsafe { &mut *(lparam.0 as *mut Vec<WindowInfo>) };

    if let Ok(info) = get_window_info(hwnd.0 as isize) {
        windows.push(info);
    }

    TRUE
}

pub fn get_window_info(hwnd: isize) -> Result<WindowInfo> {
    let handle = HWND(hwnd as *mut c_void);

    unsafe {
        let title_len = GetWindowTextLengthW(handle);
        let mut title = String::new();
        if title_len > 0 {
            let mut buf = vec![0u16; (title_len + 1) as usize];
            GetWindowTextW(handle, &mut buf);
            title = String::from_utf16_lossy(&buf[..title_len as usize]);
        }

        let mut class_buf = [0u16; 256];
        let class_len = GetClassNameW(handle, &mut class_buf);
        let class_name = String::from_utf16_lossy(&class_buf[..class_len as usize]);

        let mut rect = RECT::default();
        GetWindowRect(handle, &mut rect).ok();

        let visible = IsWindowVisible(handle).as_bool();

        let mut pid: u32 = 0;
        GetWindowThreadProcessId(handle, Some(&mut pid));

        Ok(WindowInfo {
            hwnd,
            title,
            class_name,
            rect: (rect.left, rect.top, rect.right, rect.bottom),
            visible,
            pid,
        })
    }
}

pub fn focus_window(hwnd: isize) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        SetForegroundWindow(handle).ok().context("SetForegroundWindow failed")?;
    }
    Ok(())
}

pub fn move_window(hwnd: isize, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        MoveWindow(handle, x, y, w, h, TRUE).ok().context("MoveWindow failed")?;
    }
    Ok(())
}

pub fn resize_window(hwnd: isize, w: i32, h: i32) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        let mut rect = RECT::default();
        GetWindowRect(handle, &mut rect).ok().context("GetWindowRect failed")?;
        MoveWindow(handle, rect.left, rect.top, w, h, TRUE)
            .ok()
            .context("MoveWindow failed")?;
    }
    Ok(())
}

pub fn show_window(hwnd: isize, cmd: i32) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        let _ = ShowWindow(handle, SHOW_WINDOW_CMD(cmd));
    }
    Ok(())
}

pub fn close_window(hwnd: isize) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    unsafe {
        PostMessageW(handle, WM_CLOSE, WPARAM(0), LPARAM(0))
            .ok()
            .context("PostMessage(WM_CLOSE) failed")?;
    }
    Ok(())
}

pub fn set_z_order(hwnd: isize, insert_after: isize) -> Result<()> {
    let handle = HWND(hwnd as *mut c_void);
    let insert = HWND(insert_after as *mut c_void);
    unsafe {
        SetWindowPos(handle, insert, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE)
            .ok()
            .context("SetWindowPos (Z-order) failed")?;
    }
    Ok(())
}
