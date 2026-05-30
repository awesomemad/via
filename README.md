# VIA — Sovereign GUI Engine

**VIA** (Latin: *by way of, through*) gives AI agents total control over the Windows GUI — clicking buttons, reading UI text, typing into fields, moving windows, switching virtual desktops. One engine, three adapter surfaces, every agent.

```
Agent → [MCP | CLI | C-ABI] → via-engine → Win32 API + UIA + COM → Windows
```

| Adapter | Binary | Covers |
|---------|--------|--------|
| **MCP** | `via-mcp.exe` | opencode, Hermes Agent, OpenClaw, Claude Code, Cursor, Claude Desktop, VS Code Copilot, Zed, Windsurf, Cline, Continue.dev, any MCP host |
| **CLI** | `via.exe` | Shell scripts, agents without MCP, quick testing |
| **C-ABI** | `via_plugin.dll` | C/Python/Rust/.NET hosts via `LoadLibrary` |

---

## Quick Start

```powershell
# One-command build
setup.bat

# Smoke test
.\target\release\via.exe dump
```

Output: `target/release/via.exe` (1.5 MB), `via-mcp.exe` (1.9 MB), `via_plugin.dll` (1.8 MB), `via_engine.lib`.

---

## Build

```powershell
setup.bat                                   # everything
cargo build --release                       # everything (individual)
cargo build -p via-mcp --release            # just MCP server
cargo build -p via-cli --release            # just CLI
cargo build -p via-cdylib --release         # just C-ABI plugin
```

---

## Configure Every Platform

All platforms point at the same binary: `target/release/via-mcp.exe`. The config
files below are **already in this repo** for the platforms marked ✓ — just build
and they work.

| Platform | Config file(s) | In repo | 
|----------|---------------|---------|
| Hermes Agent | `%LOCALAPPDATA%/via/plugins/via/` | ✓ `via-hermes-plugin/via/` |
| Hermes Agent (alt) | `~/.hermes/config.yaml` | |
| opencode | `.opencode/opencode.json` | ✓ |
| Claude Code | `.mcp.json` + `.claude/settings.json` | ✓ |
| OpenClaw | `openclaw.json` | ✓ |
| Cursor | Cursor Settings → MCP | |
| Claude Desktop | `claude_desktop_config.json` | |
| VS Code / Cline | `settings.json` or Cline MCP config | |

---

### Hermes Agent (Nous Research)

**Option A — Native plugin** (one command):

```powershell
install-plugin.bat
hermes plugins enable via
```

The install script:
1. Builds `via-mcp.exe`
2. Creates `%LOCALAPPDATA%\via\plugins\via\`
3. Copies `__init__.py` + `plugin.yaml` + `via-mcp.exe` into it

Resulting directory layout:

```
%LOCALAPPDATA%\via\plugins\via\
├── __init__.py       # MCP subprocess bridge
├── plugin.yaml       # Hermes manifest
└── via-mcp.exe       # bundled binary (the plugin spawns this)
```

The plugin searches for the binary in this order:

| Priority | Where it looks |
|----------|----------------|
| 1 | `$env:VIA_MCP_PATH` — explicit override |
| 2 | Same directory as `__init__.py` (bundled install) |
| 3 | `%LOCALAPPDATA%\via\plugins\via\via-mcp.exe` |
| 4 | `~\.hermes\plugins\via\via-mcp.exe` |
| 5 | `via-mcp.exe` on `$env:PATH` |

To override with a custom binary path without reinstalling:

```powershell
$env:VIA_MCP_PATH = "D:\custom\path\via-mcp.exe"
```

**Option B — MCP config** (simpler, no plugin):

```yaml
# ~/.hermes/config.yaml
mcp_servers:
  via:
    command: D:/Omnis/via/target/release/via-mcp.exe
```

---

### opencode

```json
{
  "mcp": {
    "via": {
      "command": ["./target/release/via-mcp.exe"],
      "enabled": true,
      "type": "local",
      "description": "Windows GUI automation engine"
    }
  }
}
```

Already configured at `.opencode/opencode.json` in this repo.

---

### Claude Code

```powershell
# Per-project (creates .mcp.json)
claude mcp add via -- .\target\release\via-mcp.exe

# Or add to ~/.claude/settings.json (global)
claude mcp add -s user via -- .\target\release\via-mcp.exe
```

Config files `.mcp.json` and `.claude/settings.json` are already in the repo.

---

### OpenClaw

```json
{
  "mcp": {
    "servers": {
      "via": {
        "command": "./target/release/via-mcp.exe"
      }
    }
  }
}
```

Already configured at `openclaw.json` in this repo. Or register via CLI:

```powershell
openclaw mcp set via "{"""command""": """./target/release/via-mcp.exe"""}"
```

---

### Cursor

`Cursor Settings → MCP → Add New MCP Server`:

| Field | Value |
|-------|-------|
| Name | `via` |
| Type | `stdio` |
| Command | `D:\Omnis\via\target\release\via-mcp.exe` |

---

### Claude Desktop

Edit `claude_desktop_config.json`:

| OS | Path |
|----|------|
| Windows | `%APPDATA%\Claude\claude_desktop_config.json` |

```json
{
  "mcpServers": {
    "via": {
      "command": "D:\\Omnis\\via\\target\\release\\via-mcp.exe",
      "type": "stdio",
      "description": "Windows GUI automation engine"
    }
  }
}
```

---

### Any MCP Host

Same pattern — stdio transport, point to `via-mcp.exe`:

```
command: <absolute-or-relative-path>/target/release/via-mcp.exe
type: stdio
```

---

## CLI (no MCP needed)

```powershell
via.exe dump                        # UIA tree snapshot
via.exe windows                     # list all windows
via.exe focus 0x1000c               # focus window
via.exe click 960 540               # left click
via.exe click 960 540 right         # right click
via.exe type "hello world"          # type text
via.exe key 0x1B                    # press/release key
via.exe close 0x1000c               # close window
via.exe move 0x1000c 0 0 800 600    # move + resize
```

---

## Tools (MCP)

### gui_dump

No parameters. Returns a compressed semantic map:

```
; VIA GUI MAP
; type name [state...] x1,y1,x2,y2 [h=hwnd] [p=pid]
wind Calculator 0,0,400,600 h=0x1000c p=1234
  pane Number pad 0,40,400,600 p=1234
    btn 7 0,40,100,130 p=1234
    btn 8 100,40,200,130 p=1234
```

**Element types:** `wind` (window), `btn` (button), `edit` (text field),
`lnk` (link), `pane` (container), `list` (list), `ttlb` (title bar),
`menu` (menu), `chk` (checkbox), `rad` (radio), `cmb` (combo), `tbl` (table),
`txt` (text), `sep` (screen/separator), `img` (image), `bar` (scroll/toolbar),
`tree` (tree), `tip` (tooltip), `doc` (document), `cust` (custom).

**State flags:** `+fcs` (focused), `+enabled`, `+visible`, `+off` (offscreen).

Indentation = tree depth. Max 5000 elements at 20 levels.

### gui_command

| Action | Required | Optional |
|--------|----------|----------|
| `click` | `x`, `y` | `button`: `"left"` (default), `"right"`, `"middle"` |
| `double_click` | `x`, `y` | |
| `type` | `text` | |
| `move_mouse` | `x`, `y` | |
| `focus` | `hwnd` | |
| `minimize` | `hwnd` | |
| `maximize` | `hwnd` | |
| `restore` | `hwnd` | |
| `close` | `hwnd` | |
| `hide` | `hwnd` | |
| `show` | `hwnd` | |
| `move_window` | `hwnd`, `x`, `y` | `w`, `h` (omit for move-only) |
| `resize` | `hwnd`, `w`, `h` | |
| `enum_windows` | | |
| `get_window_info` | `hwnd` | |
| `switch_desktop` | | `direction`: `1` (right), `-1` (left) |
| `key_press` | `vk` | |
| `key_release` | `vk` | |
| `post_message` | `hwnd`, `msg` | `wparam`, `lparam` |

**hwnd**: decimal `268435468`, hex string `"0x1000c"`, or raw hex `0x1000c`.

**vk**: virtual-key code (decimal or hex). Common ones:

| Key | VK | Key | VK | Key | VK |
|-----|-----|-----|-----|-----|-----|
| Enter | `0x0D` | Tab | `0x09` | Escape | `0x1B` |
| Backspace | `0x08` | Delete | `0x2E` | Space | `0x20` |
| Ctrl | `0x11` | Shift | `0x10` | Alt | `0x12` |
| Left | `0x25` | Up | `0x26` | Right | `0x27` |
| Down | `0x28` | Home | `0x24` | End | `0x23` |
| F1–F10 | `0x70`–`0x79` | A–Z | `0x41`–`0x5A` | 0–9 | `0x30`–`0x39` |

### Keyboard shortcuts (key down + key up)

```python
# Ctrl+C (copy)
gui_command action=key_press   params={vk: 0x11}   # Ctrl down
gui_command action=key_press   params={vk: 0x43}   # C down
gui_command action=key_release params={vk: 0x43}   # C up
gui_command action=key_release params={vk: 0x11}   # Ctrl up
```

---

## Workflow

```
1. gui_dump              ← inspect the screen
2. gui_command action=X  ← act
3. gui_dump              ← verify
```

### Click a button

```
gui_dump → find "btn OK 500,400,600,430"
gui_command action=click params={x: 550, y: 415}
```

### Type into a text field

```
gui_dump → find "edit" and its coords
gui_command action=click params={x: <center_x>, y: <center_y>}
gui_command action=type  params={text: "hello world"}
```

### Focus and resize a window

```
gui_dump → find the window HWND
gui_command action=focus       params={hwnd: "0x1000c"}
gui_command action=move_window params={hwnd: "0x1000c", x: 0, y: 0, w: 800, h: 600}
```

### Close a window

```
gui_command action=close params={hwnd: "0x1000c"}
```

### Switch virtual desktop

```
gui_command action=switch_desktop params={direction: 1}   # right
gui_command action=switch_desktop params={direction: -1}  # left
```

---

## C-ABI (via_plugin.dll)

For hosts that load native plugins (Python, C, Rust, .NET).

### C signature

```c
int     via_create();               // init COM + UIA (call once)
void    via_destroy();              // shutdown
char*   via_dump();                 // UIA tree — free with via_free_string
char*   via_enum_windows();         // JSON window list
char*   via_command(action, json);  // execute, returns JSON result
void    via_free_string(char*);     // free returned strings
```

### Python (ctypes)

```python
import ctypes
dll = ctypes.CDLL("via_plugin.dll")
dll.via_create()

ptr = dll.via_dump()
dump = ctypes.cast(ptr, ctypes.c_char_p).value.decode()
dll.via_free_string(ptr)

cmd = dll.via_command(b"click", b'{"x":960,"y":540}')
result = ctypes.cast(cmd, ctypes.c_char_p).value.decode()
dll.via_free_string(cmd)

dll.via_destroy()
```

### Rust

```rust
#[link(name = "via_plugin")]
extern "C" {
    fn via_create() -> i32;
    fn via_dump() -> *mut c_char;
    fn via_command(action: *const c_char, params: *const c_char) -> *mut c_char;
    fn via_free_string(s: *mut c_char);
    fn via_destroy();
}
```

---

## Architecture

```
via-engine/       Core — Win32/UIA/COM, zero agent awareness
├── via-cli/      CLI adapter        → via.exe         (1.5 MB)
├── via-mcp/      MCP protocol       → via-mcp.exe     (1.9 MB)
├── via-cdylib/   C-ABI plugin       → via_plugin.dll  (1.8 MB)
├── via-hermes-plugin/  Hermes plugin  → %LOCALAPPDATA%/via/plugins/via/
│
├── .opencode/    opencode MCP config
├── .claude/      Claude Code MCP config
├── .mcp.json     Claude Code / generic MCP config
├── openclaw.json OpenClaw MCP config
│
├── AGENTS.md     Agent instructions
└── README.md     This file
```

Each adapter crate is ~200 lines of protocol glue depending on `via-engine`.
The engine never knows what agent is calling it.

---

## Troubleshooting

### "via-mcp.exe not found"

Build it first:
```powershell
setup.bat
```

### Hermes plugin fails to load

Run the installer again to refresh all files:

```powershell
install-plugin.bat
```

If it still fails, check the installed files:

```powershell
ls $env:LOCALAPPDATA\via\plugins\via\
```

Should show `__init__.py`, `plugin.yaml`, and `via-mcp.exe`. Missing any file?
Re-run `install-plugin.bat`.

Override the binary path to confirm the engine works:

```powershell
$env:VIA_MCP_PATH = "D:\Omnis\via\target\release\via-mcp.exe"
# Now restart Hermes and retry
```

### COM initialization fails

VIA calls `CoInitializeEx` once at startup. If the calling process has already
initialized COM in a conflicting mode, `via_create()` returns -1. This usually
doesn't happen — the engine handles it.

### UIA dump returns nothing

Run the CLI directly to isolate:
```powershell
.\target\release\via.exe dump
```
If that works, the engine is fine — the issue is in the MCP/Hermes wiring.

### Process stays alive after Hermes exits

Kill it manually:
```powershell
Stop-Process -Name via-mcp -Force
```

### no tools appear in the MCP host

- Verify `via-mcp.exe` starts: run it directly and send `{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}`
- Check the host's MCP server list: most hosts have a `/mcp` command or settings panel
- Restart the host after adding the server

---

## Why VIA?

- **Single path** — one engine for every agent, no more SendKeys/pyautogui/AutoIt
- **No runtime** — 1.5 MB standalone binary, zero Python/.NET/Node deps
- **AI-native** — compressed UIA map designed for LLM consumption
- **Total coverage** — UIA tree, window management, virtual desktops, raw input, PostMessage
- **Future-proof** — MCP today, C-ABI for tomorrow's native plugin hosts
