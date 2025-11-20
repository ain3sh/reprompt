\# Technical Spec: `reprompt` (Prompt Trim)



\*\*Version:\*\* 1.0

\*\*Target Binary Name:\*\* `reprompt` (or `reprompt.exe`)

\*\*Primary Goal:\*\* A single-file executable CLI tool that sanitizes system clipboard content by removing TUI (Text User Interface) artifacts (bounding boxes, borders, padding) while preserving actual content formatting (newlines, indentation, code syntax).

\*\*Target Stack:\*\* Rust (Latest Stable)



---



\## 1. Architecture \& Dependencies



We choose \*\*Rust\*\* for its ability to compile to a static binary with zero runtime dependencies, its speed, and its robust string handling.



\### 1.1 Crate Selection (Dependencies)

Add these to `Cargo.toml`:

\*   \*\*`arboard`\*\*: The standard for cross-platform clipboard interaction (handles Cocoa, X11/Wayland, WinAPI).

\*   \*\*`regex`\*\*: For robust pattern matching.

\*   \*\*`lazy\_static`\*\* or \*\*`once\_cell`\*\*: To compile regex patterns once at runtime initialization.

\*   \*\*`is\_wsl`\*\*: To detect if running inside Windows Subsystem for Linux (requires special handling).

\*   \*\*`anyhow`\*\*: For clean, idiomatic error propagation.



\### 1.2 Cross-Platform Strategy

The code must handle three "modes" of operation:

1\.  \*\*Native Windows/macOS:\*\* Use `arboard` directly.

2\.  \*\*Native Linux (Desktop):\*\* Use `arboard` (requires X11 or Wayland presence).

3\.  \*\*WSL2 (Headless Linux):\*\* This is the critical edge case. `arboard` will fail because there is no display server. We must proxy clipboard commands to the Windows host executables.



---



\## 2. Implementation Logic



\### 2.1 The Main Loop

1\.  \*\*Detect OS Environment\*\*: Check if we are in WSL2 or a standard OS.

2\.  \*\*Read Clipboard\*\*: Fetch string data. Handle empty/non-string data gracefully (exit silently or print error).

3\.  \*\*Process Text\*\*: Run the cleaning heuristics.

4\.  \*\*Compare\*\*: If `cleaned\_text == original\_text`, exit (do nothing to save write cycles).

5\.  \*\*Write Clipboard\*\*: Push the cleaned string back to system clipboard.

6\.  \*\*Feedback\*\*: Print a small success message (e.g., `✨`) to stdout.



\### 2.2 The Cleaning Algorithm (The "Meat")

We must remove "Box Drawing" characters defined in Unicode standards, but \*\*only\*\* when they act as borders. We must not remove pipes `|` used in code (e.g., logical OR, bitwise OR, Bash pipes).



\*\*Target Artifacts:\*\*

\*   Vertical borders: `│`, `║`, `|` (only if matched at start/end of line).

\*   Horizontal borders: `─`, `═`, `━`.

\*   Corners: `╭`, `╮`, `╯`, `╰`, `┌`, `┐`, `└`, `┘`.



\#### Regex Logic

We need two main patterns.



\*\*Pattern A: Pure Border Lines\*\*

Lines that serve purely as decoration (top/bottom of box) should be removed entirely.

\*   \*\*Regex:\*\* `^\[\\s╭╮╰╯─═━┌┐└┘]+$`

\*   \*\*Action:\*\* If a line matches this completely, delete the line.



\*\*Pattern B: Content Wrappers\*\*

Lines that contain content surrounded by borders.

\*   \*\*Regex Breakdown:\*\*

&nbsp;   1.  `^\\s\*` (Start of line, optional indentation)

&nbsp;   2.  `\[│║]` (The border character)

&nbsp;   3.  `\[ ]?` (Optional single padding space often added by TUIs)

&nbsp;   4.  `(?P<content>.\*?)` (Lazy capture of the actual content)

&nbsp;   5.  `\[ ]?` (Optional single padding space)

&nbsp;   6.  `\[│║]?` (Optional trailing border—optional because some copies miss the very end)

&nbsp;   7.  `\\s\*$` (End of line)

\*   \*\*Action:\*\* Extract `<content>`.



\*\*The "Safety Check" (Heuristic)\*\*

To prevent destroying Markdown Tables or Code, apply this logic:

\*   Iterate through all lines.

\*   If a line matches \*\*Pattern A\*\*, discard it.

\*   If a line matches \*\*Pattern B\*\*, replace it with the capture group.

\*   If a line matches \*\*neither\*\* (e.g., it looks like normal text), keep it exactly as is.



\### 2.3 WSL2 Bridge Implementation

WSL2 cannot access the Windows clipboard via standard Linux APIs. You must shell out.



\*\*Reading in WSL2:\*\*

Execute `powershell.exe` to get the clipboard.

```rust

Command::new("powershell.exe")

&nbsp;   .args(\&\["-NoProfile", "-Command", "Get-Clipboard"])

&nbsp;   .output()

```

\*Note: PowerShell `Get-Clipboard` may return `\\r\\n` line endings. Ensure normalization to `\\n` before processing.\*



\*\*Writing in WSL2:\*\*

Pipe data into `clip.exe`.

```rust

let mut child = Command::new("clip.exe").stdin(Stdio::piped()).spawn()?;

child.stdin.as\_mut().unwrap().write\_all(data.as\_bytes())?;

```

\*Note: `clip.exe` is a Windows binary. It expects UTF-16 LE in some environments, but usually accepts UTF-8 text via pipe fine. Test this.\*



---



\## 3. Development Resources \& References



\*   \*\*Regex Testing:\*\* Use \[Regex101](https://regex101.com/) to validate the patterns against the examples below. Set the flavor to PCRE or Rust.

\*   \*\*Box Drawing Characters:\*\* \[Wikipedia: Box-drawing character](https://en.wikipedia.org/wiki/Box-drawing\_character). Refer to this if you encounter exotic border styles.

\*   \*\*Crate Docs:\*\*

&nbsp;   \*   \[arboard](https://docs.rs/arboard/latest/arboard/)

&nbsp;   \*   \[regex](https://docs.rs/regex/latest/regex/)



---



\## 4. Test Cases (Ground Truth)



Use these inputs to verify the Regex logic.



\*\*Case 1: The Standard Agent Box\*\*

\*Input:\*

```text

╭─────────────────────────────────────────────────────────╮

│ > This is an example of how things                      │

│   might go very wrong.                                  │

╰─────────────────────────────────────────────────────────╯

```

\*Expected Output:\*

```text

> This is an example of how things

&nbsp; might go very wrong.

```

\*(Note: The indent on line 2 should be preserved relative to the border removal).\*



\*\*Case 2: Code with Pipes (The Danger Zone)\*\*

\*Input:\*

```text

│ let x = a | b; │

```

\*Expected Output:\*

```text

let x = a | b;

```

\*(The internal pipe must remain).\*



\*\*Case 3: Markdown Table (Do Not Destroy)\*\*

\*Input:\*

```text

| Header 1 | Header 2 |

| -------- | -------- |

| Data A   | Data B   |

```

\*Expected Output:\*

```text

| Header 1 | Header 2 |

| -------- | -------- |

| Data A   | Data B   |

```

\*(The regex must NOT match this because the internal structure is complex, or we must explicitly decide that if it looks like a Markdown table, we skip it. \*\*Refinement:\*\* Markdown tables usually don't have spaces \*outside\* the pipes. TUI boxes usually have indentation. If `reprompt` strips the outer pipes of a markdown table, it breaks the table. \*\*Decision:\*\* If the regex logic is strict (`^ \*│ .\* │ \*$`), it might strip the outer pipes of a table. However, restoring a prompt usually implies extracting text. If the user copies a table inside a TUI box, the TUI box is the outer layer. For V1, prioritize stripping outer wrappers.)\*



---



\## 5. Build \& Delivery Instructions



\### Compilation

The user will need to build for their target.

\*   \*\*Development:\*\* `cargo run`

\*   \*\*Release Build:\*\* `cargo build --release`



\### User Installation Guide (Draft)

\*To be included in the README.\*



1\.  \*\*Windows:\*\*

&nbsp;   \*   Place `reprompt.exe` in a folder in your `%PATH%`.

&nbsp;   \*   Create a shortcut to the exe. Right-click shortcut -> Properties -> "Shortcut Key". Set to `Ctrl+Alt+V` (or similar).

2\.  \*\*macOS:\*\*

&nbsp;   \*   Move binary to `/usr/local/bin/reprompt`.

&nbsp;   \*   Use \*\*Automator\*\*: Create a "Quick Action" -> "Run Shell Script" (`/usr/local/bin/reprompt`) -> Save as "Trim Clipboard".

&nbsp;   \*   Assign Keybind in System Settings -> Keyboard -> Shortcuts -> Services.

3\.  \*\*Linux / WSL2:\*\*

&nbsp;   \*   Move binary to `/usr/local/bin/reprompt`.

&nbsp;   \*   Add alias to `.bashrc` or bind via your Terminal Emulator settings.



---



\## 6. Implementation Checklist



1\.  \[ ] \*\*Setup:\*\* `cargo init`. Add dependencies.

2\.  \[ ] \*\*WSL Check:\*\* Implement `is\_wsl\_custom()` (wrapper around `is\_wsl` crate or manual `/proc/version` check) early in `main`.

3\.  \[ ] \*\*Clipboard Read:\*\* Implement `get\_clipboard()` handling `arboard` vs `powershell`.

4\.  \[ ] \*\*Regex Logic:\*\* Implement `clean\_text(input: \&str) -> String`. Use the test cases above as unit tests in `main.rs` (`#\[test]`).

5\.  \[ ] \*\*Clipboard Write:\*\* Implement `set\_clipboard()` handling `arboard` vs `clip.exe`.

6\.  \[ ] \*\*Performance:\*\* Ensure Regex is compiled once (use `lazy\_static!`).

7\.  \[ ] \*\*Build:\*\* Verify `cargo build --release` produces a binary < 5MB (approx).

8\.  \[ ] \*\*Verify:\*\* Test copying text from a terminal TUI, running the binary, and pasting into VS Code.

