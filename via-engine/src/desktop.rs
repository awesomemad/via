use std::ffi::c_void;

use anyhow::{Context, Result};
use windows::Win32::Foundation::{BOOL, HWND};
use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VIRTUAL_KEY};

const CLSID_VIRTUAL_DESKTOP_MANAGER: windows::core::GUID = windows::core::GUID::from_u128(0xAA509086_5CA9_4C0C_9B8C_8F6D5A1B9D4E);
const IID_IVIRTUAL_DESKTOP_MANAGER: windows::core::GUID = windows::core::GUID::from_u128(0x4CE81583_1E4C_4564_A7E3_AD25F2C2B1D2);

const VK_LWIN: u16 = 0x5B;
const VK_LCONTROL: u16 = 0xA2;
const VK_LEFT: u16 = 0x25;
const VK_RIGHT: u16 = 0x27;

#[repr(C)]
struct IVirtualDesktopManagerVtbl {
    query_interface: unsafe extern "system" fn(*mut c_void, *const windows::core::GUID, *mut *mut c_void) -> i32,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    is_window_on_current_virtual_desktop: unsafe extern "system" fn(*mut c_void, HWND, *mut BOOL) -> i32,
    get_window_desktop_id: unsafe extern "system" fn(*mut c_void, HWND, *mut windows::core::GUID) -> i32,
    move_window_to_desktop: unsafe extern "system" fn(*mut c_void, HWND, *const windows::core::GUID) -> i32,
}

#[link(name = "ole32")]
extern "system" {
    fn CoCreateInstance(
        rclsid: *const windows::core::GUID,
        pUnkOuter: *mut c_void,
        dwClsContext: u32,
        riid: *const windows::core::GUID,
        ppv: *mut *mut c_void,
    ) -> i32;
}

static COM_INITIALIZED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

pub fn init_com() -> Result<()> {
    COM_INITIALIZED.get_or_init(|| {
        unsafe { let _ = CoInitializeEx(None, COINIT_MULTITHREADED); }
        true
    });
    Ok(())
}

fn create_virtual_desktop_manager() -> Result<*mut c_void> {
    unsafe {
        let mut manager: *mut c_void = std::ptr::null_mut();
        let hr = CoCreateInstance(
            &CLSID_VIRTUAL_DESKTOP_MANAGER,
            std::ptr::null_mut(),
            1u32, // CLSCTX_LOCAL_SERVER
            &IID_IVIRTUAL_DESKTOP_MANAGER,
            &mut manager,
        );
        if hr < 0 || manager.is_null() {
            return Err(anyhow::anyhow!("IVirtualDesktopManager unavailable (hr={:#x})", hr));
        }
        Ok(manager)
    }
}

fn release_com(ptr: *mut c_void) {
    if !ptr.is_null() {
        unsafe {
            let vtbl = *(ptr as *mut *const IVirtualDesktopManagerVtbl);
            ((*vtbl).release)(ptr);
        }
    }
}

pub fn move_window_to_desktop(hwnd: isize, desktop_guid_str: &str) -> Result<()> {
    let manager = create_virtual_desktop_manager()?;
    let handle = HWND(hwnd as *mut c_void);
    let desktop_id = parse_guid(desktop_guid_str)
        .context("Invalid GUID format. Use: {XXXXXXXX-XXXX-XXXX-XXXX-XXXXXXXXXXXX}")?;

    let vtbl = unsafe { *(manager as *mut *const IVirtualDesktopManagerVtbl) };
    let hr = unsafe { ((*vtbl).move_window_to_desktop)(manager, handle, &desktop_id) };

    release_com(manager);

    if hr < 0 {
        return Err(anyhow::anyhow!("move_window_to_desktop failed: {:#x}", hr));
    }
    Ok(())
}

pub fn is_window_on_current_desktop(hwnd: isize) -> Result<bool> {
    let manager = create_virtual_desktop_manager()?;
    let handle = HWND(hwnd as *mut c_void);

    let vtbl = unsafe { *(manager as *mut *const IVirtualDesktopManagerVtbl) };
    let mut result = BOOL::default();
    let hr = unsafe { ((*vtbl).is_window_on_current_virtual_desktop)(manager, handle, &mut result) };

    release_com(manager);

    if hr < 0 {
        return Err(anyhow::anyhow!("is_window_on_current_virtual_desktop failed: {:#x}", hr));
    }
    Ok(result.as_bool())
}

pub fn switch_desktop(direction: i32) -> Result<()> {
    let arrow = if direction < 0 { VK_LEFT } else { VK_RIGHT };
    let keys = [VK_LWIN, VK_LCONTROL, arrow];
    let mut inputs = Vec::new();

    for &vk in &keys {
        inputs.push(make_key_input(VIRTUAL_KEY(vk), KEYBD_EVENT_FLAGS(0)));
    }
    for &vk in keys.iter().rev() {
        inputs.push(make_key_input(VIRTUAL_KEY(vk), KEYEVENTF_KEYUP));
    }

    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32); }
    Ok(())
}

pub fn make_key_input(w_vk: VIRTUAL_KEY, dw_flags: KEYBD_EVENT_FLAGS) -> INPUT {
    let mut input: INPUT = unsafe { std::mem::zeroed() };
    input.r#type = INPUT_KEYBOARD;
    input.Anonymous.ki.wVk = w_vk;
    input.Anonymous.ki.wScan = 0;
    input.Anonymous.ki.dwFlags = dw_flags;
    input.Anonymous.ki.time = 0;
    input.Anonymous.ki.dwExtraInfo = 0;
    input
}

fn parse_guid(s: &str) -> Option<windows::core::GUID> {
    let s = s.trim().trim_start_matches('{').trim_end_matches('}');
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() != 5 { return None; }

    let data1 = u32::from_str_radix(parts[0], 16).ok()?;
    let data2 = u16::from_str_radix(parts[1], 16).ok()?;
    let data3 = u16::from_str_radix(parts[2], 16).ok()?;

    let combined = format!("{}{}", parts[3], parts[4]);
    let bytes = combined.as_bytes();
    if bytes.len() != 16 { return None; }
    let mut data4 = [0u8; 8];
    for i in 0..8 {
        let hex = std::str::from_utf8(&bytes[i * 2..i * 2 + 2]).ok()?;
        data4[i] = u8::from_str_radix(hex, 16).ok()?;
    }

    Some(windows::core::GUID { data1, data2, data3, data4 })
}
