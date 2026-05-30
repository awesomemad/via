use anyhow::{Context, Result};
use windows::Win32::Foundation::RECT;
use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_INPROC_SERVER};
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationElement, TreeScope_Children,
};

const CLSID_CUIAUTOMATION: windows::core::GUID = windows::core::GUID::from_u128(0xff48dba4_60ef_4201_aa87_54103eef594e);

fn control_type_name(ctrl_id: i32) -> &'static str {
    match ctrl_id {
        50000 => "btn",
        50001 => "chk",
        50002 => "cmb",
        50003 => "edt",
        50004 => "lnk",
        50005 => "img",
        50006 => "li",
        50007 => "lst",
        50008 => "mnu",
        50009 => "mnb",
        50010 => "mni",
        50011 => "prg",
        50012 => "rad",
        50013 => "scrl",
        50014 => "sld",
        50015 => "spn",
        50016 => "stb",
        50017 => "tab",
        50018 => "tabi",
        50019 => "txt",
        50020 => "tlb",
        50021 => "tip",
        50022 => "tre",
        50023 => "trei",
        50024 => "grp",
        50025 => "thm",
        50026 => "doc",
        50027 => "sbtn",
        50028 => "win",
        50029 => "pne",
        50030 => "hdr",
        50031 => "tbl",
        50032 => "ttlb",
        50033 => "sep",
        50034 => "appb",
        50035 => "cal",
        50036 => "cust",
        _ => "?",
    }
}

fn safe_str(b: &windows::core::BSTR) -> String {
    if b.is_empty() { String::new() } else { b.to_string() }
}

fn rect_fmt(r: &RECT) -> String {
    format!("{},{},{},{}", r.left, r.top, r.right, r.bottom)
}

fn get_uia() -> Result<IUIAutomation> {
    unsafe { CoCreateInstance(&CLSID_CUIAUTOMATION, None, CLSCTX_INPROC_SERVER).context("CoCreateInstance IUIAutomation") }
}

fn crawl(
    el: &IUIAutomationElement,
    uia: &IUIAutomation,
    depth: usize,
    out: &mut String,
    max_d: usize,
    remaining: &mut u32,
) {
    if depth > max_d || *remaining == 0 { return; }
    *remaining -= 1;

    let indent = "  ".repeat(depth);

    let ct = unsafe { el.CurrentControlType() }
        .map(|ct| ct.0)
        .unwrap_or(-1);
    let name = unsafe { el.CurrentName() }
        .map(|b| safe_str(&b))
        .unwrap_or_default();
    let aid = unsafe { el.CurrentAutomationId() }
        .map(|b| safe_str(&b))
        .unwrap_or_default();
    let r = unsafe { el.CurrentBoundingRectangle() }
        .unwrap_or(RECT::default());
    let enabled = unsafe { el.CurrentIsEnabled() }
        .map(|b| b.as_bool())
        .unwrap_or(true);
    let focused = unsafe { el.CurrentHasKeyboardFocus() }
        .map(|b| b.as_bool())
        .unwrap_or(false);
    let offscreen = unsafe { el.CurrentIsOffscreen() }
        .map(|b| b.as_bool())
        .unwrap_or(true);
    let pid: i32 = unsafe { el.CurrentProcessId() }
        .unwrap_or(0);
    let hw: isize = unsafe { el.CurrentNativeWindowHandle() }
        .map(|hw| hw.0 as isize)
        .unwrap_or(0);

    let role = control_type_name(ct);

    let mut parts = vec![role.to_string()];

    if !name.is_empty() {
        parts.push(name);
    } else if !aid.is_empty() {
        parts.push(format!("#{}", aid));
    }

    if focused { parts.push("+fcs".to_string()); }
    if !enabled { parts.push("!dis".to_string()); }
    if offscreen { parts.push("~off".to_string()); }
    if r.right > r.left || r.bottom > r.top { parts.push(rect_fmt(&r)); }
    if hw != 0 { parts.push(format!("h=0x{:x}", hw)); }
    if pid != 0 { parts.push(format!("p={}", pid)); }

    out.push_str(&format!("{}{}\n", indent, parts.join(" ")));

    if let Ok(cond) = unsafe { uia.CreateTrueCondition() } {
        if let Ok(arr) = unsafe { el.FindAll(TreeScope_Children, &cond) } {
            let len = unsafe { arr.Length().unwrap_or(0) };
            for i in 0..len {
                if *remaining == 0 { break; }
                if let Ok(child) = unsafe { arr.GetElement(i) } {
                    crawl(&child, uia, depth + 1, out, max_d, remaining);
                }
            }
        }
    }
}

pub fn dump_ui_tree() -> Result<String> {
    let uia = get_uia()?;
    let root = unsafe { uia.GetRootElement().context("GetRootElement")? };

    let mut out = String::from("; VIA GUI MAP\n; type name [state...] x1,y1,x2,y2 [h=hwnd] [p=pid]\n");
    let mut remaining = 5000u32;
    crawl(&root, &uia, 0, &mut out, 20, &mut remaining);

    if remaining == 0 {
        out.push_str("; ... truncated at 5000 elements\n");
    }

    Ok(out)
}
