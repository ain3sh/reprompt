use anyhow::{Context, Result};
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

    // ANSI escape codes (colors, formatting) that TUIs often add
    static ref RE_ANSI: Regex = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]").expect("Invalid ANSI Regex");
}

/// Represents a clipboard transaction with rollback capability
struct ClipboardTransaction {
    original: String,
    modified: Option<String>,
}

impl ClipboardTransaction {
    /// Creates a new transaction by reading the current clipboard
    fn new() -> Result<Self> {
        let original = get_clipboard().context("Failed to read clipboard for transaction")?;
        Ok(Self {
            original,
            modified: None,
        })
    }

    /// Gets the original clipboard content
    fn original(&self) -> &str {
        &self.original
    }

    /// Sets the modified content (doesn't commit yet)
    fn set_modified(&mut self, modified: String) {
        self.modified = Some(modified);
    }

    /// Validates that the modified content is not corrupted
    fn validate(&self) -> Result<()> {
        let modified = self.modified.as_ref()
            .ok_or_else(|| anyhow::anyhow!("No modified content to validate"))?;

        // Bail on Unicode replacement character (U+FFFD indicates encoding corruption)
        if modified.contains('\u{FFFD}') {
            anyhow::bail!("Unicode replacement character (U+FFFD) detected");
        }

        // Sanity check: if original had substantial content but cleaned is empty,
        // we likely over-cleaned (false positive on content detection)
        let original_has_content = self.original.trim().len() > 10;
        let cleaned_is_empty = modified.trim().is_empty();

        if original_has_content && cleaned_is_empty {
            anyhow::bail!("Cleaning removed all content (likely false positive)");
        }

        // Sanity check: if cleaned text is dramatically shorter (>90% reduction),
        // and original was substantial, we might have over-cleaned
        if self.original.len() > 200 && modified.len() < self.original.len() / 10 {
            eprintln!("Warning: Cleaning reduced content by >90% ({} -> {} bytes)",
                     self.original.len(), modified.len());
            eprintln!("This might indicate over-aggressive cleaning.");
        }

        Ok(())
    }

    /// Commits the transaction by writing to clipboard with validation
    fn commit(self) -> Result<()> {
        let modified = self.modified
            .ok_or_else(|| anyhow::anyhow!("No modified content to commit"))?;

        // If content is identical, skip write
        if modified == self.original {
            return Ok(());
        }

        // Attempt to write with proper encoding
        if let Err(e) = set_clipboard(&modified) {
            // Attempt rollback on write failure
            eprintln!("Write failed: {}. Attempting rollback...", e);
            if let Err(rollback_err) = set_clipboard(&self.original) {
                eprintln!("CRITICAL: Rollback failed: {}", rollback_err);
                eprintln!("Original clipboard content may be lost!");
                return Err(anyhow::anyhow!(
                    "Write failed and rollback failed: {} -> {}",
                    e,
                    rollback_err
                ));
            }
            eprintln!("Rollback successful. Clipboard restored to original state.");
            return Err(anyhow::anyhow!("Transaction aborted: {}", e));
        }

        // Verify the write by reading back
        match get_clipboard() {
            Ok(readback) => {
                // Normalize both strings for comparison to handle platform differences
                // (PowerShell might add trailing newline, etc.)
                let expected_normalized = modified.trim_end();
                let readback_normalized = readback.trim_end();

                if readback_normalized != expected_normalized {
                    eprintln!("Verification failed: Clipboard content doesn't match expected result");
                    eprintln!("Expected {} bytes, got {} bytes",
                             expected_normalized.len(), readback_normalized.len());
                    eprintln!("Attempting rollback...");
                    if let Err(rollback_err) = set_clipboard(&self.original) {
                        eprintln!("CRITICAL: Rollback failed: {}", rollback_err);
                        return Err(anyhow::anyhow!("Verification and rollback both failed"));
                    }
                    eprintln!("Rollback successful.");
                    return Err(anyhow::anyhow!("Transaction aborted: Verification failed"));
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not verify write: {}", e);
                eprintln!("Clipboard may have been updated, but verification failed.");
            }
        }

        Ok(())
    }

}

/// Checks if the program is running inside WSL.
fn is_wsl_custom() -> bool {
    is_wsl::is_wsl()
}

/// Reads text from the system clipboard with proper encoding handling.
/// Handles Native (arboard) and WSL (powershell) environments.
fn get_clipboard() -> Result<String> {
    if is_wsl_custom() {
        // Try PowerShell first (WSL interop) with explicit UTF-8 encoding
        match Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Get-Clipboard"
            ])
            .output()
        {
            Ok(output) if output.status.success() => {
                // Explicitly handle UTF-8 decoding
                let text = match String::from_utf8(output.stdout) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Warning: Invalid UTF-8 from PowerShell, using lossy conversion");
                        String::from_utf8_lossy(&e.into_bytes()).to_string()
                    }
                };

                // Normalize line endings from CRLF to LF
                let normalized = text.replace("\r\n", "\n");

                // Trim trailing whitespace that PowerShell often adds
                let trimmed = normalized.trim_end().to_string();

                return Ok(trimmed);
            }
            Ok(output) => {
                // PowerShell ran but failed
                return Err(anyhow::anyhow!(
                    "PowerShell Get-Clipboard failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // powershell.exe not found - WSL interop likely disabled
                // Fall back to arboard
                eprintln!("Warning: WSL detected but powershell.exe not found.");
                eprintln!("Windows interop may be disabled. Falling back to native clipboard.");
                eprintln!("To fix: Check /etc/wsl.conf has [interop] enabled=true");
                let mut clipboard = arboard::Clipboard::new()?;
                return Ok(clipboard.get_text()?);
            }
            Err(e) => {
                // Other error running powershell.exe
                return Err(e.into());
            }
        }
    } else {
        let mut clipboard = arboard::Clipboard::new()?;
        Ok(clipboard.get_text()?)
    }
}

/// Writes text to the system clipboard with proper encoding handling.
/// Handles Native (arboard) and WSL (clip.exe) environments.
fn set_clipboard(data: &str) -> Result<()> {
    if is_wsl_custom() {
        // Use PowerShell Set-Clipboard for better encoding support
        // clip.exe can have encoding issues with UTF-8
        // CRITICAL: Set InputEncoding to UTF-8 for pipeline input to prevent mojibake
        match Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "[Console]::InputEncoding = [System.Text.Encoding]::UTF8; $input | Set-Clipboard"
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(mut child) => {
                {
                    let mut stdin = child.stdin.take()
                        .ok_or_else(|| anyhow::anyhow!("Failed to open stdin for PowerShell"))?;

                    // Write UTF-8 bytes directly
                    // PowerShell will handle the encoding correctly
                    stdin.write_all(data.as_bytes())
                        .context("Failed to write to PowerShell stdin")?;

                    // Explicitly drop stdin to close the pipe and signal EOF
                    drop(stdin);
                }

                let output = child.wait_with_output()
                    .context("Failed to wait for PowerShell process")?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    return Err(anyhow::anyhow!("PowerShell Set-Clipboard failed: {}", stderr));
                }

                Ok(())
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                // powershell.exe not found - try clip.exe as fallback (ASCII-only)
                // clip.exe writes raw UTF-8 bytes which can garble non-ASCII text
                if data.is_ascii() {
                    eprintln!("Warning: powershell.exe not found, trying clip.exe...");

                    match Command::new("clip.exe")
                        .stdin(Stdio::piped())
                        .spawn()
                    {
                        Ok(mut child) => {
                            let mut stdin = child.stdin.take()
                                .ok_or_else(|| anyhow::anyhow!("Failed to open stdin for clip.exe"))?;

                            stdin.write_all(data.as_bytes())?;
                            drop(stdin);

                            let status = child.wait()?;
                            if !status.success() {
                                return Err(anyhow::anyhow!("clip.exe failed"));
                            }
                            return Ok(());
                        }
                        Err(_) => {
                            // clip.exe not available, fall through to arboard
                        }
                    }
                }

                // PowerShell failed and either data is non-ASCII or clip.exe unavailable
                // Fall back to native clipboard (arboard)
                eprintln!("Warning: WSL detected but Windows interop not available.");
                eprintln!("Falling back to native clipboard.");
                eprintln!("To fix: Check /etc/wsl.conf has [interop] enabled=true");
                let mut clipboard = arboard::Clipboard::new()?;
                clipboard.set_text(data)?;
                Ok(())
            }
            Err(e) => {
                // Other error spawning powershell.exe
                Err(e.into())
            }
        }
    } else {
        let mut clipboard = arboard::Clipboard::new()?;
        clipboard.set_text(data)?;
        Ok(())
    }
}

/// Cleans the input text by removing TUI artifacts (borders, ANSI codes).
fn clean_text(input: &str) -> String {
    // First pass: strip ANSI escape codes (colors, cursor movement, etc.)
    // Many TUI applications add these for visual formatting
    let ansi_stripped = RE_ANSI.replace_all(input, "");

    let mut output = String::new();
    let mut first = true;
    let mut consecutive_empty = 0;

    for line in ansi_stripped.lines() {
        // Check if this is a pure border line (top/bottom of box)
        if RE_BORDER_LINE.is_match(line) {
            continue;
        }

        // Check if this is a content line wrapped in borders
        if let Some(caps) = RE_CONTENT_WRAPPER.captures(line) {
            if let Some(content) = caps.name("content") {
                let content_str = content.as_str();

                // Only trim trailing spaces (TUI padding), preserve leading spaces (indentation)
                // trim_end() removes the padding spaces that TUIs add to reach the right border
                let trimmed = content_str.trim_end();

                // Track consecutive empty lines to avoid bloat (apply limit globally)
                if trimmed.is_empty() {
                    consecutive_empty += 1;
                    if consecutive_empty > 2 {
                        continue; // Skip excessive empty lines from wrapped content too
                    }
                } else {
                    consecutive_empty = 0;
                }

                if !first {
                    output.push('\n');
                }
                output.push_str(trimmed);
                first = false;
            }
        } else {
            // Line doesn't match any TUI pattern - preserve as-is
            // This handles regular text, markdown, code, etc.

            // Limit consecutive empty lines to avoid bloat from TUI spacing
            if line.trim().is_empty() {
                consecutive_empty += 1;
                if consecutive_empty > 2 {
                    continue; // Skip excessive empty lines
                }
            } else {
                consecutive_empty = 0;
            }

            if !first {
                output.push('\n');
            }
            output.push_str(line);
            first = false;
        }
    }

    // Final cleanup: remove any trailing whitespace the TUI might have added
    output.trim_end().to_string()
}

fn main() -> Result<()> {
    // Phase 1: SNAPSHOT - Create transaction and backup clipboard
    let mut transaction = match ClipboardTransaction::new() {
        Ok(tx) => tx,
        Err(e) => {
            // If we cannot read clipboard, exit gracefully
            eprintln!("Error reading clipboard: {}", e);
            return Ok(());
        }
    };

    let original_text = transaction.original();

    // Handle empty clipboard gracefully
    if original_text.trim().is_empty() {
        return Ok(());
    }

    // Phase 2: TRANSFORM - Clean the text (remove TUI artifacts)
    let cleaned_text = clean_text(original_text);

    // Early exit if no changes (don't waste write cycles)
    if cleaned_text == original_text {
        return Ok(());
    }

    transaction.set_modified(cleaned_text);

    // Phase 3: VALIDATE - Check for corruption before committing
    if let Err(e) = transaction.validate() {
        eprintln!("Validation failed: {e}");
        eprintln!("Aborting operation. Clipboard unchanged.");
        return Ok(());
    }

    // Phase 4 & 5: COMMIT and VERIFY - Write with automatic verification and rollback
    match transaction.commit() {
        Ok(()) => {
            // Success feedback
            println!("✨");
            Ok(())
        }
        Err(e) => {
            eprintln!("Transaction failed: {}", e);
            // The transaction already attempted rollback
            Ok(())
        }
    }
}
