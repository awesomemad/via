"""VIA GUI Engine — Hermes native plugin.

Spawns via-mcp.exe as a long-lived subprocess and communicates via
MCP JSON-RPC over stdio. More reliable than ctypes DLL loading.

Binary search order:
  1. $VIA_MCP_PATH environment variable (explicit override)
  2. Same directory as this .py file (bundled install)
  3. %LOCALAPPDATA%/via/plugins/via/via-mcp.exe (Windows AppData)
  4. ~/.hermes/plugins/via/via-mcp.exe (Unix-style home dir)
  5. via-mcp.exe on PATH
"""

from pathlib import Path
import json
import os
import subprocess
import threading


_mcp_process = None
_mcp_lock = threading.Lock()
_request_id = 0


def _find_binary() -> str:
    env = os.environ.get("VIA_MCP_PATH")
    if env:
        p = Path(env)
        if p.exists():
            return str(p.resolve())

    candidates = [
        Path(__file__).parent / "via-mcp.exe",
        Path(os.environ.get("LOCALAPPDATA", "")) / "via" / "plugins" / "via" / "via-mcp.exe",
        Path.home() / ".hermes" / "plugins" / "via" / "via-mcp.exe",
    ]
    for p in candidates:
        if p.exists():
            return str(p.resolve())

    import shutil
    which = shutil.which("via-mcp.exe")
    if which:
        return which

    raise RuntimeError(
        "Cannot find via-mcp.exe.\n"
        "  Set $env:VIA_MCP_PATH to the full path, or\n"
        "  Install: run install-plugin.bat, or\n"
        "  Build:   setup.bat"
    )


def _send_raw(method: str, params: dict = None) -> dict:
    """Send JSON-RPC message. Caller must hold _mcp_lock."""
    global _request_id
    _request_id += 1
    req = {
        "jsonrpc": "2.0",
        "id": _request_id,
        "method": method,
        "params": params or {},
    }
    line = json.dumps(req)
    _mcp_process.stdin.write(line + "\n")
    _mcp_process.stdin.flush()
    resp_line = _mcp_process.stdout.readline()
    if not resp_line:
        raise RuntimeError("via-mcp.exe closed stdout unexpectedly")
    resp = json.loads(resp_line)
    if "error" in resp:
        raise RuntimeError(f"MCP error: {resp['error']}")
    return resp.get("result", {})


def _ensure_process():
    """Start via-mcp.exe if not running. Caller must NOT hold _mcp_lock.
    
    Safe to call from register() (single-threaded init path) or from
    _mcp_send() which holds _mcp_lock — we never try to acquire it here.
    """
    global _mcp_process
    if _mcp_process is not None and _mcp_process.poll() is None:
        return
    binary = _find_binary()
    _mcp_process = subprocess.Popen(
        [binary],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        bufsize=1,
    )
    _send_raw("initialize", {
        "protocolVersion": "2025-03-26",
        "capabilities": {},
        "clientInfo": {"name": "hermes-plugin-via", "version": "0.1.0"},
    })


def _mcp_send(method: str, params: dict = None) -> dict:
    """Thread-safe MCP message send."""
    with _mcp_lock:
        _ensure_process()
        return _send_raw(method, params)


def _mcp_tool_call(tool_name: str, args: dict) -> str:
    result = _mcp_send("tools/call", {"name": tool_name, "arguments": args})
    content = result.get("content", [])
    for block in content:
        if block.get("type") == "text":
            return block["text"]
    return json.dumps(result)


def _handle_dump(params: dict, **kwargs) -> str:
    return _mcp_tool_call("gui_dump", {})


def _handle_command(params: dict, **kwargs) -> str:
    return _mcp_tool_call("gui_command", params)


def _cleanup():
    """Kill the subprocess if running. No lock needed — only called from shutdown hook."""
    global _mcp_process
    if _mcp_process and _mcp_process.poll() is None:
        _mcp_process.terminate()
        try:
            _mcp_process.wait(timeout=5)
        except subprocess.TimeoutExpired:
            _mcp_process.kill()
            _mcp_process.wait()
    _mcp_process = None


def register(ctx):
    try:
        _ensure_process()
    except Exception as e:
        raise RuntimeError(f"VIA plugin cannot start via-mcp.exe: {e}")

    ctx.register_tool(
        name="gui_dump",
        toolset="via",
        description="Returns a compressed semantic map of the entire Windows GUI state via UIA.",
        schema={
            "name": "gui_dump",
            "description": "Returns a compressed semantic map of the entire Windows GUI state via UIA.",
            "parameters": {"type": "object", "properties": {}},
        },
        handler=_handle_dump,
    )

    ctx.register_tool(
        name="gui_command",
        toolset="via",
        description="Universal GUI command handler for Windows automation.",
        schema={
            "name": "gui_command",
            "description": "Universal GUI command handler for Windows automation.",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "description": (
                            "Action: click | double_click | type | move_mouse | "
                            "focus | minimize | maximize | restore | close | hide | show | "
                            "move_window | resize | enum_windows | get_window_info | "
                            "switch_desktop | key_press | key_release | post_message"
                        ),
                    },
                    "params": {
                        "type": "object",
                        "description": "Parameters for the action.",
                    },
                },
                "required": ["action"],
            },
        },
        handler=_handle_command,
    )

    ctx.register_hook("on_session_end", lambda **kw: _cleanup())
