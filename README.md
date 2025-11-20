# reprompt

`reprompt` is a single-file executable CLI tool that sanitizes system clipboard content by removing TUI (Text User Interface) artifacts (bounding boxes, borders, padding) while preserving actual content formatting (newlines, indentation, code syntax).

It is designed to help when copying text from terminal UIs (like AI agents, logs, etc.) that use box-drawing characters.

## Installation

### Single Line Install (Linux / macOS / WSL)

```bash
curl -fsSL https://ain3sh.com/reprompt/install.sh | bash
```
equivalent to
```bash
curl -fsSL https://raw.githubusercontent.com/ain3sh/reprompt/main/scripts/install.sh | bash
```

### Manual Installation

Download the latest release for your platform from the [Releases Page](https://github.com/ain3sh/reprompt/releases).

- **Linux**: `reprompt-linux-amd64`
- **macOS**: `reprompt-macos-amd64`
- **Windows**: `reprompt-windows-amd64.exe`

Place the binary in your `$PATH` (e.g., `~/.local/bin` or `/usr/local/bin` on Linux/macOS).

### Build from Source

Requires [Rust](https://rustup.rs/).

```bash
git clone https://github.com/ain3sh/reprompt.git
cd reprompt
cargo build --release
# Binary will be in target/release/reprompt
```

## Usage

Simply run the `reprompt` command. It will:
1. Read your current clipboard content.
2. Remove TUI borders and artifacts.
3. Write the cleaned text back to the clipboard.
4. Print `✨` to the console if successful.

```bash
reprompt
```

### Recommended Workflow

#### Windows
1. Place `reprompt.exe` in a folder in your `%PATH%`.
2. Create a shortcut to the exe.
3. Right-click shortcut -> Properties -> "Shortcut Key". Set to `Ctrl+Alt+V` (or similar).

#### macOS
1. Move binary to a folder in your PATH (e.g. `~/.local/bin/reprompt`).
2. Use **Automator**: Create a "Quick Action" -> "Run Shell Script" (`path/to/reprompt`) -> Save as "Trim Clipboard".
3. Assign Keybind in System Settings -> Keyboard -> Shortcuts -> Services.

#### Linux / WSL
Add an alias or bind a key in your terminal emulator or window manager.

```bash
# Example alias
alias cleanclip='reprompt'
```

## Supported Environments & Dependencies

| Platform | Environment | Runtime Requirements | Notes |
| :--- | :--- | :--- | :--- |
| **Windows** | Native | None | Works out of the box. |
| **macOS** | Native | None | Works out of the box. |
| **WSL** | WSL2 | `powershell.exe`, `clip.exe` (optional) | Proxies to Windows clipboard. Falls back to native if interop disabled. |
| **Linux** | Desktop (X11/Wayland) | X Server / Wayland | Requires a clipboard manager (usually built-in) for persistence. |
| **Linux** | Headless (CI/SSH) | `xvfb`, `x11-apps` | Requires `xvfb` for display and `xclipboard` (from `x11-apps`) to persist data after exit. |

## Troubleshooting

### WSL2: "Error reading clipboard: No such file or directory (os error 2)"

This error occurs when WSL2 detects your environment as WSL but cannot access Windows executables (`powershell.exe`, `clip.exe`). This is a **known issue with WSL2 + systemd** on Ubuntu 24.04 LTS and newer distributions.

**Root Cause:** When systemd is enabled in WSL2, it creates `/proc/sys/fs/binfmt_misc/WSLInterop-late` instead of `/proc/sys/fs/binfmt_misc/WSLInterop`, breaking Windows interop.

**Solution 1: Enable WSL Interop** (Recommended)

1. Check if interop is disabled:
   ```bash
   cat /etc/wsl.conf
   ```

2. Ensure `/etc/wsl.conf` contains:
   ```ini
   [interop]
   enabled=true
   appendWindowsPath=true
   ```

3. Restart WSL from PowerShell/CMD:
   ```powershell
   wsl --shutdown
   ```

4. Reopen your WSL terminal and test:
   ```bash
   powershell.exe -Command "echo test"
   ```

**Solution 2: Use Automatic Fallback**

`reprompt` automatically falls back to native Linux clipboard (`arboard`) when Windows executables are not accessible. Simply ensure you have a display server (X11/Wayland) running.

**Solution 3: Create Systemd Service**

For persistent fix with systemd, create `/etc/systemd/system/wsl-interop-fix.service`:

```ini
[Unit]
Description=Fix WSL Interop for Windows executables
After=systemd-binfmt.service

[Service]
Type=oneshot
ExecStart=/bin/bash -c 'echo ":WSLInterop:M::MZ::/init:" > /proc/sys/fs/binfmt_misc/register'
RemainAfterExit=yes

[Install]
WantedBy=multi-user.target
```

Enable it:
```bash
sudo systemctl enable wsl-interop-fix.service
sudo systemctl start wsl-interop-fix.service
```

### Linux: "Error reading clipboard: X11 server connection timed out"

This occurs on headless Linux systems (SSH, CI/CD) without a display server.

**Solution:**
- Install and run `xvfb`:
  ```bash
  sudo apt install xvfb x11-apps
  Xvfb :99 -screen 0 1024x768x24 &
  export DISPLAY=:99
  reprompt
  ```
