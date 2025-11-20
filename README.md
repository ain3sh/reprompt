# reprompt

A transaction-based clipboard sanitizer that removes TUI artifacts from copied text while preserving the content you actually want.

## The Problem

When you copy text from terminal UIs (AI agents, logs, monitoring dashboards), you get this:

```
╭─────────────────────────────────────────────╮
│ def fibonacci(n):                           │
│     if n <= 1:                              │
│         return n                            │
│     return fibonacci(n-1) + fibonacci(n-2)  │
╰─────────────────────────────────────────────╯
```

But you wanted this:

```python
def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
```

## The Solution

Run `reprompt` after copying. It strips borders, ANSI color codes, and padding while preserving indentation and structure. If anything goes wrong, it automatically rolls back to your original clipboard.

```bash
reprompt  # ✨
```

## Installation

**One-liner (Linux/macOS/WSL):**
```bash
curl -fsSL https://ain3sh.com/reprompt/install.sh | bash
```

**Pre-built binaries:** [Releases Page](https://github.com/ain3sh/reprompt/releases)

**From source:**
```bash
cargo install --git https://github.com/ain3sh/reprompt
```

## How It Works

`reprompt` uses a 5-phase transaction model with automatic rollback:

1. **Snapshot** — Backs up your clipboard before any modification
2. **Transform** — Strips borders, ANSI codes, and excessive whitespace
3. **Validate** — Detects encoding corruption (mojibake) before writing
4. **Commit** — Writes cleaned text with proper UTF-8 handling
5. **Verify** — Reads back and compares; rolls back if mismatch detected

This architecture prevents data loss even when clipboard operations fail or introduce corruption.

## Platform Support

| Platform | Requirements | Notes |
|----------|--------------|-------|
| **macOS** | None | Native clipboard support |
| **Windows** | None | Native clipboard support |
| **Linux** | X11/Wayland | Desktop environments work out of the box |
| **WSL2** | PowerShell interop | Falls back to native if interop disabled |

## Key Features

- **Zero data loss** — Original clipboard backed up before modification
- **Encoding-safe** — Detects and prevents UTF-8 corruption (mojibake)
- **ANSI stripping** — Removes terminal color codes automatically
- **Smart validation** — Won't destroy your clipboard with aggressive cleaning
- **Graceful degradation** — Falls back through multiple clipboard methods

## Advanced Usage

### Keyboard Shortcuts

**macOS:**
```bash
# 1. Create Automator Quick Action
# 2. Add "Run Shell Script": /path/to/reprompt
# 3. System Settings → Keyboard → Shortcuts → assign key
```

**Linux:**
```bash
# Bind to your window manager
bindsym $mod+Shift+v exec reprompt
```

**Windows:**
Create a shortcut with `Ctrl+Alt+V` assigned in Properties → Shortcut Key.

## Troubleshooting

**WSL2: "Error reading clipboard"**

WSL interop may be disabled. Fix:
```bash
# Add to /etc/wsl.conf
[interop]
enabled=true
appendWindowsPath=true

# Restart WSL
wsl.exe --shutdown
```

**Linux headless: "X11 server connection timed out"**

SSH/CI environments need a virtual display:
```bash
sudo apt install xvfb x11-apps
Xvfb :99 -screen 0 1024x768x24 &
export DISPLAY=:99
```

## Technical Details

- **Language:** Rust (static binary, no runtime dependencies)
- **Binary size:** ~3.4MB
- **Clipboard backend:** `arboard` (cross-platform) + PowerShell (WSL)
- **Architecture:** Transaction-based with ACID-like properties

## License

MIT
