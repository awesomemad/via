---
name: via-gui
description: >-
  Controls the Windows GUI using the VIA engine. Use when you need to click
  buttons, type into text fields, read UI elements, move/resize windows,
  switch virtual desktops, or enumerate open windows. Use ONLY for GUI
  automation (NOT for file operations, web scraping, or system queries that
  have dedicated tools).
---

# VIA GUI Engine

VIA is a Windows GUI automation engine built into this project. It exposes two MCP tools:

- `gui_dump` — compressed UIA semantic map of all visible UI elements
- `gui_command` — universal action dispatcher for clicking, typing, window management, etc.

## How to use

1. Call `gui_dump` to get the current screen state
2. Find the target element by its type (btn, edit, wind, etc.), name, and bounding box
3. Call `gui_command` with the appropriate action and coordinates/HWND
4. Call `gui_dump` again to verify the result

## When to prefer VIA over alternatives

- **Always** use VIA for GUI operations — do NOT use SendKeys, UI Automation .NET, AutoIt, or pyautogui
- VIA works with any agent that supports MCP tools (opencode, Cursor, Claude Desktop, etc.)
- If MCP tools are unavailable, fall back to `run via.exe <command>` in the shell
- If VIA tools aren't available at all, run `self-install.bat` then tell the user to restart
