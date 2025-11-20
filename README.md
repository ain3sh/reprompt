# reprompt

`reprompt` is a single-file executable CLI tool that sanitizes system clipboard content by removing TUI (Text User Interface) artifacts (bounding boxes, borders, padding) while preserving actual content formatting (newlines, indentation, code syntax).

It is designed to help when copying text from terminal UIs (like AI agents, logs, etc.) that use box-drawing characters.

## Installation

### Single Line Install (Linux / macOS / WSL)

```bash
curl -fsSL https://raw.githubusercontent.com/ain3sh/reprompt/main/scripts/install.sh | bash
```

### Manual Installation

Download the latest release for your platform from the [Releases Page](https://github.com/ain3sh/reprompt/releases).

- **Linux**: `reprompt-linux-amd64`
- **macOS**: `reprompt-macos-amd64`
- **Windows**: `reprompt-windows-amd64.exe`

Place the binary in your `$PATH` (e.g., `/usr/local/bin` on Linux/macOS).

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
1. Move binary to `/usr/local/bin/reprompt`.
2. Use **Automator**: Create a "Quick Action" -> "Run Shell Script" (`/usr/local/bin/reprompt`) -> Save as "Trim Clipboard".
3. Assign Keybind in System Settings -> Keyboard -> Shortcuts -> Services.

#### Linux / WSL
Add an alias or bind a key in your terminal emulator or window manager.

```bash
# Example alias
alias cleanclip='reprompt'
```

## Supported Environments

- **Linux (X11/Wayland)**: Uses `arboard`.
- **Windows**: Uses `arboard`.
- **macOS**: Uses `arboard`.
- **WSL2 (Headless)**: Automatically detects WSL and proxies clipboard commands to Windows host via `powershell.exe` and `clip.exe`.
