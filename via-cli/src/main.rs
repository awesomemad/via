use via_engine::OmniseyeEngine;

fn main() -> anyhow::Result<()> {
    let engine = OmniseyeEngine::new()?;

    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("VIA - Sovereign GUI Engine v0.1");
        eprintln!("Usage:");
        eprintln!("  via dump                              - Dump UIA tree");
        eprintln!("  via windows                           - List all windows");
        eprintln!("  via focus <hwnd>                      - Focus window");
        eprintln!("  via move <hwnd> <x> <y> <w> <h>      - Move/resize window");
        eprintln!("  via close <hwnd>                      - Close window");
        eprintln!("  via click <x> <y> [left|right|middle] - Click at position");
        eprintln!("  via type <text>                       - Type text");
        eprintln!("  via key <vk>                          - Press and release key");
        return Ok(());
    }

    match args[1].as_str() {
        "dump" => {
            let dump = engine.dump_ui_tree()?;
            println!("{}", dump);
        }
        "windows" => {
            for w in engine.enum_windows()? {
                println!(
                    "0x{:x} {:?} visible={} rect={},{},{},{} pid={}",
                    w.hwnd, w.title, w.visible, w.rect.0, w.rect.1, w.rect.2, w.rect.3, w.pid
                );
            }
        }
        "focus" => {
            let hwnd = parse_hwnd(&args[2])?;
            engine.focus_window(hwnd)?;
            println!("Focused 0x{:x}", hwnd);
        }
        "move" => {
            let hwnd = parse_hwnd(&args[2])?;
            let x: i32 = args[3].parse()?;
            let y: i32 = args[4].parse()?;
            let w: i32 = args[5].parse()?;
            let h: i32 = args[6].parse()?;
            engine.move_window(hwnd, x, y, w, h)?;
            println!("Moved 0x{:x}", hwnd);
        }
        "close" => {
            let hwnd = parse_hwnd(&args[2])?;
            engine.close_window(hwnd)?;
            println!("Closed 0x{:x}", hwnd);
        }
        "click" => {
            let x: i32 = args[2].parse()?;
            let y: i32 = args[3].parse()?;
            let button = if args.len() > 4 {
                match args[4].as_str() {
                    "right" => via_engine::input::MouseButton::Right,
                    "middle" => via_engine::input::MouseButton::Middle,
                    _ => via_engine::input::MouseButton::Left,
                }
            } else {
                via_engine::input::MouseButton::Left
            };
            engine.click(x, y, button)?;
            println!("Clicked ({},{})", x, y);
        }
        "type" => {
            let text = args[2..].join(" ");
            engine.type_text(&text)?;
            println!("Typed: {}", text);
        }
        "key" => {
            let vk = u16::from_str_radix(args[2].trim_start_matches("0x"), 16).unwrap_or(0);
            engine.key_press(vk)?;
            std::thread::sleep(std::time::Duration::from_millis(50));
            engine.key_release(vk)?;
            println!("Sent key 0x{:x}", vk);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
        }
    }

    Ok(())
}

fn parse_hwnd(s: &str) -> anyhow::Result<isize> {
    if s.starts_with("0x") || s.starts_with("0X") {
        Ok(isize::from_str_radix(&s[2..], 16)?)
    } else {
        Ok(s.parse::<isize>()?)
    }
}
