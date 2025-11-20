use anyhow::Result;
use regex::Regex;
use lazy_static::lazy_static;
use std::process::{Command, Stdio};
use std::io::Write;

lazy_static! {
    static ref RE_BORDER_LINE: Regex = Regex::new(r"^[\s╭╮╰╯─═━┌┐└┘]+$").expect("Invalid Border Line Regex");
    static ref RE_CONTENT_WRAPPER: Regex = Regex::new(r"(?x)
        ^
        \s*           # Start of line, optional indentation
        [│║]          # The border character
        \x20?         # Optional single padding space
        (?P<content>.*?) # Lazy capture of the actual content
        \x20?         # Optional single padding space
        [│║]?         # Optional trailing border
        \s*           # End of line
        $
    ").expect("Invalid Content Wrapper Regex");
}

/// Checks if the program is running inside WSL.
fn is_wsl_custom() -> bool {
    is_wsl::is_wsl()
}

/// Reads text from the system clipboard.
/// Handles Native (arboard) and WSL (powershell) environments.
fn get_clipboard() -> Result<String> {
    if is_wsl_custom() {
        let output = Command::new("powershell.exe")
            .args(&["-NoProfile", "-Command", "Get-Clipboard"])
            .output()?;

        if !output.status.success() {
            return Err(anyhow::anyhow!("PowerShell Get-Clipboard failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let text = String::from_utf8_lossy(&output.stdout).to_string();
        // Normalize line endings from CRLF to LF
        Ok(text.replace("\r\n", "\n"))
    } else {
        let mut clipboard = arboard::Clipboard::new()?;
        Ok(clipboard.get_text()?)
    }
}

/// Writes text to the system clipboard.
/// Handles Native (arboard) and WSL (clip.exe) environments.
fn set_clipboard(data: &str) -> Result<()> {
    if is_wsl_custom() {
        let mut child = Command::new("clip.exe")
            .stdin(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdin for clip.exe"))?;
        stdin.write_all(data.as_bytes())?;
        drop(stdin); // Close stdin to signal EOF

        let status = child.wait()?;
        if !status.success() {
             return Err(anyhow::anyhow!("clip.exe failed"));
        }
        Ok(())
    } else {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(data)?;
        Ok(())
    }
}

/// Cleans the input text by removing TUI artifacts.
fn clean_text(input: &str) -> String {
    let mut output = String::new();
    let mut first = true;

    for line in input.lines() {
        if RE_BORDER_LINE.is_match(line) {
            continue;
        } else if let Some(caps) = RE_CONTENT_WRAPPER.captures(line) {
            if let Some(content) = caps.name("content") {
                if !first {
                    output.push('\n');
                }
                output.push_str(content.as_str().trim_end());
                first = false;
            }
        } else {
            // Match neither - keep line as is
            if !first {
                output.push('\n');
            }
            output.push_str(line);
            first = false;
        }
    }
    output
}

fn main() -> Result<()> {
    // 2. Read Clipboard
    let original_text = match get_clipboard() {
        Ok(text) => text,
        Err(e) => {
            // Spec says: exit silently or print error.
            // If we are in a headless environment without clipboard support, this will likely fail.
            // We print to stderr for debugging but don't panic.
            eprintln!("Error reading clipboard: {}", e);
            return Ok(());
        }
    };

    if original_text.trim().is_empty() {
        // Handle empty data gracefully
        return Ok(());
    }

    // 3. Process Text
    let cleaned_text = clean_text(&original_text);

    // 4. Compare
    if cleaned_text == original_text {
        // exit (do nothing to save write cycles)
        return Ok(());
    }

    // 5. Write Clipboard
    if let Err(e) = set_clipboard(&cleaned_text) {
        eprintln!("Error writing clipboard: {}", e);
        return Ok(());
    }

    // 6. Feedback
    println!("✨");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_1_standard_agent_box() {
        let input = "\
╭─────────────────────────────────────────────────────────╮
│ > This is an example of how things                      │
│   might go very wrong.                                  │
╰─────────────────────────────────────────────────────────╯";
        let expected = "\
> This is an example of how things
  might go very wrong.";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_case_2_code_with_pipes() {
        let input = "│ let x = a | b; │";
        let expected = "let x = a | b;";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_case_3_markdown_table() {
        let input = "\
| Header 1 | Header 2 |
| -------- | -------- |
| Data A   | Data B   |";
        let expected = "\
| Header 1 | Header 2 |
| -------- | -------- |
| Data A   | Data B   |";
        assert_eq!(clean_text(input), expected);
    }

    #[test]
    fn test_mixed_content() {
        let input = "\
╭────────╮
│ line 1 │
│ line 2 │
╰────────╯
normal line";
        let expected = "\
line 1
line 2
normal line";
        assert_eq!(clean_text(input), expected);
    }
}
