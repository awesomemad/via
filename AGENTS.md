# VIA — Sovereign GUI Engine

This project builds a Windows GUI automation engine for AI agents. The engine
exposes total GUI control through three adapter surfaces:

## Adapters / How to use

| Surface | What | How the agent calls it |
|---------|------|------------------------|
| **MCP** | `gui_dump` + `gui_command` tools | Native tools in registry — use them directly like any built-in tool |
| **CLI** | `via.exe dump \| windows \| focus \| click \| type \| key \| close \| move` | `run via.exe dump` via shell |
| **C-ABI** | `via_plugin.dll` | `LoadLibrary` + `GetProcAddress` from any C/Rust/Python host |

## When to use VIA

- **Always** for any GUI interaction — clicking buttons, reading UI text,
  typing into fields, moving/resizing windows, switching virtual desktops,
  enumerating open windows.
- **Never** use SendKeys, UI Automation from .NET, AutoIt, or pyautogui —
  VIA is the sole GUI automation path.
- Replace ad-hoc mouse/keyboard simulation with MCP tools whenever possible.
  Fall back to CLI only if MCP is unavailable.

## Workflow

1. **Inspect**: `gui_dump` to get the compressed UIA semantic map
2. **Parse**: Identify the element by type, name, and coordinates
3. **Act**: `gui_command` with precise coordinates or HWND
4. **Verify**: `gui_dump` again to confirm state change

## GUI Dump format

```
; VIA GUI MAP
; type name [state...] x1,y1,x2,y2 [h=hwnd] [p=pid]
wind Calculator 0,0,400,600 h=0x1000c p=1234
  btn 7 0,40,100,130 p=1234
  btn 8 100,40,200,130 p=1234
```

Indentation = tree depth. Element types: `wind` (window), `btn` (button),
`edit` (text field), `lnk` (link), `pane` (container), `list` (list),
`ttlb` (title bar), `menu` (menu), `chk` (checkbox), `rad` (radio),
`cmb` (combo), `tbl` (table), `txt` (text), `sep` (screen/separator),
`img` (image), `bar` (scroll/toolbar), `tree` (tree), `tip` (tooltip),
`doc` (document), `cust` (custom).

State flags: `+fcs` (focused), `+enabled`, `+visible`, `+off` (offscreen).

## Common GUI patterns

### Click a button

```
1. gui_dump → find btn with matching name and coords
2. gui_command action=click params={x: <center_x>, y: <center_y>}
```

### Type into a text field

```
1. gui_dump → find edit field, note its coords
2. gui_command action=click params={x: <field_center_x>, y: <field_center_y>}
3. gui_command action=type params={text: "hello world"}
```

### Focus a window by HWND

```
gui_command action=focus params={hwnd: "0x1000c"}
```

### Close a window

```
gui_command action=close params={hwnd: "0x1000c"}
```

### Switch virtual desktop

```
gui_command action=switch_desktop params={direction: 1}
```

### Move + resize a window

```
gui_command action=move_window params={hwnd: "0x1000c", x: 0, y: 0, w: 800, h: 600}
```

### Send keyboard shortcut

```
gui_command action=key_press params={vk: 0x11}   # Ctrl down
gui_command action=key_press params={vk: 0x43}   # C down
gui_command action=key_release params={vk: 0x43} # C up
gui_command action=key_release params={vk: 0x11} # Ctrl up
```

## Coordinate system

Primary display is typically 1920x1080. All coordinates are screen-absolute
pixels. HWND values are `isize` numbers — pass as hex string `"0x1000c"`,
decimal number, or raw hex number.

## CLI fallback

If MCP tools are unavailable, use the shell:

```
run via.exe dump
run via.exe click 960 540
run via.exe focus 0x1000c
run via.exe type "hello world"
```

## Self-install (if VIA tools are missing)

If `gui_dump` and `gui_command` aren't available, install VIA yourself:

```powershell
# 1. Run the self-installer
run self-install.bat

# 2. If running inside Hermes:
run hermes plugins enable via

# 3. Tell the human to restart their agent
```

The self-installer detects what platform you're on (Hermes, opencode, Claude Code,
etc.) and installs the right adapter. If your platform isn't auto-detected, just
build and point any MCP host at the binary:

```powershell
run setup.bat
# Then register via-mcp.exe as an MCP stdio server
```

## Build

```
setup.bat    # one-command release build
cargo build --release   # or build individually
```
