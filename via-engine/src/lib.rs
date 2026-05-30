pub mod window;
pub mod input;
pub mod desktop;
pub mod uia;

use anyhow::Result;

pub struct OmniseyeEngine;

impl OmniseyeEngine {
    pub fn new() -> Result<Self> {
        desktop::init_com()?;
        Ok(Self)
    }

    pub fn enum_windows(&self) -> Result<Vec<window::WindowInfo>> {
        window::enum_windows()
    }

    pub fn focus_window(&self, hwnd: isize) -> Result<()> {
        window::focus_window(hwnd)
    }

    pub fn move_window(&self, hwnd: isize, x: i32, y: i32, w: i32, h: i32) -> Result<()> {
        window::move_window(hwnd, x, y, w, h)
    }

    pub fn resize_window(&self, hwnd: isize, w: i32, h: i32) -> Result<()> {
        window::resize_window(hwnd, w, h)
    }

    pub fn minimize_window(&self, hwnd: isize) -> Result<()> {
        window::show_window(hwnd, 6)
    }

    pub fn maximize_window(&self, hwnd: isize) -> Result<()> {
        window::show_window(hwnd, 3)
    }

    pub fn restore_window(&self, hwnd: isize) -> Result<()> {
        window::show_window(hwnd, 9)
    }

    pub fn close_window(&self, hwnd: isize) -> Result<()> {
        window::close_window(hwnd)
    }

    pub fn hide_window(&self, hwnd: isize) -> Result<()> {
        window::show_window(hwnd, 0)
    }

    pub fn show_window(&self, hwnd: isize) -> Result<()> {
        window::show_window(hwnd, 5)
    }

    pub fn set_window_z_order(&self, hwnd: isize, insert_after: isize) -> Result<()> {
        window::set_z_order(hwnd, insert_after)
    }

    pub fn get_window_info(&self, hwnd: isize) -> Result<window::WindowInfo> {
        window::get_window_info(hwnd)
    }

    pub fn click(&self, x: i32, y: i32, button: input::MouseButton) -> Result<()> {
        input::click(x, y, button)
    }

    pub fn double_click(&self, x: i32, y: i32) -> Result<()> {
        input::double_click(x, y)
    }

    pub fn move_mouse(&self, x: i32, y: i32) -> Result<()> {
        input::move_mouse(x, y)
    }

    pub fn type_text(&self, text: &str) -> Result<()> {
        input::type_text(text)
    }

    pub fn key_press(&self, vk: u16) -> Result<()> {
        input::key_press(vk)
    }

    pub fn key_release(&self, vk: u16) -> Result<()> {
        input::key_release(vk)
    }

    pub fn post_message(&self, hwnd: isize, msg: u32, wparam: usize, lparam: isize) -> Result<()> {
        input::post_message(hwnd, msg, wparam, lparam)
    }

    pub fn move_window_to_desktop(&self, hwnd: isize, desktop_guid: &str) -> Result<()> {
        desktop::move_window_to_desktop(hwnd, desktop_guid)
    }

    pub fn is_window_on_current_desktop(&self, hwnd: isize) -> Result<bool> {
        desktop::is_window_on_current_desktop(hwnd)
    }

    pub fn switch_desktop(&self, direction: i32) -> Result<()> {
        desktop::switch_desktop(direction)
    }

    pub fn dump_ui_tree(&self) -> Result<String> {
        uia::dump_ui_tree()
    }
}
