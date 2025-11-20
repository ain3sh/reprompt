use anyhow::{Context, Result};
use base64::prelude::*;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::Write;
use std::process::{Command, Stdio};

lazy_static! {
    static ref RE_BORDER_LINE: Regex = Regex::new(r"^[\s╭╮╰╯─═━┌┐└┘]+$").expect("Invalid Border Line Regex");

    // Handles borders that have text embedded, e.g., "╭─── Title ───╮"
    static ref RE_TITLED_BORDER: Regex = Regex::new(r"(?x)
        ^[\s╭┌╰└]           # Start with corner or space
        (?:.*?)             # Content (title, etc.)
        [─═━]{3,}           # Must contain at least 3 horizontal bars
        (?:.*?)             # More content
        [╮┐╯┘]\s*$          # End with corner
    ").expect("Invalid Titled Border Regex");

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

    // Improved ANSI escape codes regex
    // Matches standard CSI sequences and some common others
    static ref RE_ANSI: Regex = Regex::new(r"[\x1b\x9b][\[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]").expect("Invalid ANSI Regex");
}

const WINDOWS_1252_DECODE: [char; 256] = [
    '\u{0000}', '\u{0001}', '\u{0002}', '\u{0003}', '\u{0004}', '\u{0005}', '\u{0006}', '\u{0007}',
    '\u{0008}', '\u{0009}', '\u{000A}', '\u{000B}', '\u{000C}', '\u{000D}', '\u{000E}', '\u{000F}',
    '\u{0010}', '\u{0011}', '\u{0012}', '\u{0013}', '\u{0014}', '\u{0015}', '\u{0016}', '\u{0017}',
    '\u{0018}', '\u{0019}', '\u{001A}', '\u{001B}', '\u{001C}', '\u{001D}', '\u{001E}', '\u{001F}',
    '\u{0020}', '\u{0021}', '\u{0022}', '\u{0023}', '\u{0024}', '\u{0025}', '\u{0026}', '\u{0027}',
    '\u{0028}', '\u{0029}', '\u{002A}', '\u{002B}', '\u{002C}', '\u{002D}', '\u{002E}', '\u{002F}',
    '\u{0030}', '\u{0031}', '\u{0032}', '\u{0033}', '\u{0034}', '\u{0035}', '\u{0036}', '\u{0037}',
    '\u{0038}', '\u{0039}', '\u{003A}', '\u{003B}', '\u{003C}', '\u{003D}', '\u{003E}', '\u{003F}',
    '\u{0040}', '\u{0041}', '\u{0042}', '\u{0043}', '\u{0044}', '\u{0045}', '\u{0046}', '\u{0047}',
    '\u{0048}', '\u{0049}', '\u{004A}', '\u{004B}', '\u{004C}', '\u{004D}', '\u{004E}', '\u{004F}',
    '\u{0050}', '\u{0051}', '\u{0052}', '\u{0053}', '\u{0054}', '\u{0055}', '\u{0056}', '\u{0057}',
    '\u{0058}', '\u{0059}', '\u{005A}', '\u{005B}', '\u{005C}', '\u{005D}', '\u{005E}', '\u{005F}',
    '\u{0060}', '\u{0061}', '\u{0062}', '\u{0063}', '\u{0064}', '\u{0065}', '\u{0066}', '\u{0067}',
    '\u{0068}', '\u{0069}', '\u{006A}', '\u{006B}', '\u{006C}', '\u{006D}', '\u{006E}', '\u{006F}',
    '\u{0070}', '\u{0071}', '\u{0072}', '\u{0073}', '\u{0074}', '\u{0075}', '\u{0076}', '\u{0077}',
    '\u{0078}', '\u{0079}', '\u{007A}', '\u{007B}', '\u{007C}', '\u{007D}', '\u{007E}', '\u{007F}',
    '\u{20AC}', '\u{0081}', '\u{201A}', '\u{0192}', '\u{201E}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{02C6}', '\u{2030}', '\u{0160}', '\u{2039}', '\u{0152}', '\u{008D}', '\u{017D}', '\u{008F}',
    '\u{0090}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{02DC}', '\u{2122}', '\u{0161}', '\u{203A}', '\u{0153}', '\u{009D}', '\u{017E}', '\u{0178}',
    '\u{00A0}', '\u{00A1}', '\u{00A2}', '\u{00A3}', '\u{00A4}', '\u{00A5}', '\u{00A6}', '\u{00A7}',
    '\u{00A8}', '\u{00A9}', '\u{00AA}', '\u{00AB}', '\u{00AC}', '\u{00AD}', '\u{00AE}', '\u{00AF}',
    '\u{00B0}', '\u{00B1}', '\u{00B2}', '\u{00B3}', '\u{00B4}', '\u{00B5}', '\u{00B6}', '\u{00B7}',
    '\u{00B8}', '\u{00B9}', '\u{00BA}', '\u{00BB}', '\u{00BC}', '\u{00BD}', '\u{00BE}', '\u{00BF}',
    '\u{00C0}', '\u{00C1}', '\u{00C2}', '\u{00C3}', '\u{00C4}', '\u{00C5}', '\u{00C6}', '\u{00C7}',
    '\u{00C8}', '\u{00C9}', '\u{00CA}', '\u{00CB}', '\u{00CC}', '\u{00CD}', '\u{00CE}', '\u{00CF}',
    '\u{00D0}', '\u{00D1}', '\u{00D2}', '\u{00D3}', '\u{00D4}', '\u{00D5}', '\u{00D6}', '\u{00D7}',
    '\u{00D8}', '\u{00D9}', '\u{00DA}', '\u{00DB}', '\u{00DC}', '\u{00DD}', '\u{00DE}', '\u{00DF}',
    '\u{00E0}', '\u{00E1}', '\u{00E2}', '\u{00E3}', '\u{00E4}', '\u{00E5}', '\u{00E6}', '\u{00E7}',
    '\u{00E8}', '\u{00E9}', '\u{00EA}', '\u{00EB}', '\u{00EC}', '\u{00ED}', '\u{00EE}', '\u{00EF}',
    '\u{00F0}', '\u{00F1}', '\u{00F2}', '\u{00F3}', '\u{00F4}', '\u{00F5}', '\u{00F6}', '\u{00F7}',
    '\u{00F8}', '\u{00F9}', '\u{00FA}', '\u{00FB}', '\u{00FC}', '\u{00FD}', '\u{00FE}', '\u{00FF}',
];

fn decode_windows_1252(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| WINDOWS_1252_DECODE[*b as usize])
        .collect()
}

fn encode_windows_1252(text: &str) -> Option<Vec<u8>> {
    let mut output = Vec::with_capacity(text.len());

    for ch in text.chars() {
        let byte = match ch {
            '\u{20AC}' => 0x80,
            '\u{201A}' => 0x82,
            '\u{0192}' => 0x83,
            '\u{201E}' => 0x84,
            '\u{2026}' => 0x85,
            '\u{2020}' => 0x86,
            '\u{2021}' => 0x87,
            '\u{02C6}' => 0x88,
            '\u{2030}' => 0x89,
            '\u{0160}' => 0x8A,
            '\u{2039}' => 0x8B,
            '\u{0152}' => 0x8C,
            '\u{017D}' => 0x8E,
            '\u{2018}' => 0x91,
            '\u{2019}' => 0x92,
            '\u{201C}' => 0x93,
            '\u{201D}' => 0x94,
            '\u{2022}' => 0x95,
            '\u{2013}' => 0x96,
            '\u{2014}' => 0x97,
            '\u{02DC}' => 0x98,
            '\u{2122}' => 0x99,
            '\u{0161}' => 0x9A,
            '\u{203A}' => 0x9B,
            '\u{0153}' => 0x9C,
            '\u{017E}' => 0x9E,
            '\u{0178}' => 0x9F,
            ch if (ch as u32) <= 0x7F => ch as u8,
            ch if (0x00A0..=0x00FF).contains(&(ch as u32)) => ch as u8,
            ch if (0x0080..=0x009F).contains(&(ch as u32)) => ch as u8,
            _ => return None,
        };

        output.push(byte);
    }

    Some(output)
}

/// Produces alternate interpretations of the incoming text so we can recover from
/// mojibake before stripping borders.
fn normalize_variants(input: &str) -> Vec<String> {
    let mut variants = Vec::new();

    // Remove a potential BOM and reuse as the baseline candidate
    let baseline = input.trim_start_matches('\u{feff}').to_string();
    variants.push(baseline.clone());

    // If the text looks like UTF-8 bytes that were decoded as Windows-1252
    // (common when copying from Windows apps into WSL), try to rehydrate the
    // original UTF-8. We pick this candidate only if it successfully round-trips
    // back into valid UTF-8 bytes.
    if let Some(repaired) = recover_from_cp1252_mojibake(&baseline) {
        if repaired != baseline {
            variants.push(repaired);
        }
    }

    variants
}

/// Attempts to reverse common mojibake by treating the text as if UTF-8 bytes
/// were decoded with Windows-1252 and then re-encoded to UTF-8. If that yields
/// a valid UTF-8 string, return it.
fn recover_from_cp1252_mojibake(input: &str) -> Option<String> {
    let encoded = encode_windows_1252(input)?;
    String::from_utf8(encoded).ok()
}

fn is_borderish(ch: char) -> bool {
    // True for box drawing characters, heavy line art, and characters commonly
    // seen when those glyphs are mis-decoded (Ã, â, etc.).
    // Note: '?' is NOT included despite appearing in some mojibake patterns,
    // because it's a common punctuation mark that should be preserved.
    (('\u{2500}'..='\u{257f}').contains(&ch))
        || (('\u{2580}'..='\u{259f}').contains(&ch))
        || matches!(
            ch,
            '╭' | '╮'
                | '╯'
                | '╰'
                | '╔'
                | '╗'
                | '╚'
                | '╝'
                | '╠'
                | '╣'
                | '╦'
                | '╩'
                | '╬'
                | '╪'
                | '╫'
                | '╞'
                | '╡'
                | '╥'
                | '╨'
                | '╳'
                | '│'
                | '║'
                | '─'
                | '━'
                | '═'
                | '┼'
                | '┬'
                | '┴'
                | '├'
                | '┤'
                | '┌'
                | '┐'
                | '└'
                | '┘'
                | '∩'
                | '╜'
                | '╛'
                | '╓'
                | '╖'
                | '╙'
                | '╘'
                | '╟'
                | 'â'
                | 'Ã'
                | 'ã'
                | 'Â'
                | 'ï'
                | '»'
                | '¿'
        )
}

fn is_mostly_borderish(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut borderish = 0usize;
    let mut alnum = 0usize;
    let mut printable = 0usize;
    let mut longest_border_run = 0usize;
    let mut current_border_run = 0usize;

    for ch in trimmed.chars() {
        if ch.is_control() {
            continue;
        }
        printable += 1;
        if is_borderish(ch) {
            borderish += 1;
            current_border_run += 1;
            longest_border_run = longest_border_run.max(current_border_run);
        }
        if ch.is_alphanumeric() {
            alnum += 1;
        }
        if !is_borderish(ch) {
            current_border_run = 0;
        }
    }

    if printable == 0 {
        return false;
    }

    let border_ratio_high = borderish * 4 >= printable * 3; // >=75%
    let starts_with_border = trimmed.chars().next().map(is_borderish).unwrap_or(false);
    let ends_with_border = trimmed.chars().last().map(is_borderish).unwrap_or(false);
    let border_frames_text = starts_with_border && ends_with_border && borderish > alnum;
    let border_dominates = starts_with_border && ends_with_border && borderish * 2 >= printable;
    let strong_border_run = starts_with_border
        && ends_with_border
        && longest_border_run >= 3
        && borderish * 2 > printable;

    border_ratio_high || border_frames_text || border_dominates || strong_border_run
}

fn unwrap_wrapped_line(line: &str) -> Option<String> {
    let stripped_leading = line.trim_start_matches('\u{feff}');
    if stripped_leading.is_empty() {
        return Some(String::new());
    }

    let chars: Vec<char> = stripped_leading.chars().collect();
    let len = chars.len();

    // Detect an opening run of border glyphs (after any indentation)
    let mut left = 0;
    while left < len && is_borderish(chars[left]) {
        left += 1;
    }

    if left == 0 {
        // Not wrapped with border characters on the left
        return None;
    }

    // Skip a single padding space if present after the left border
    if left < len && chars[left] == ' ' {
        left += 1;
    }

    let mut right = len;
    while right > left && is_borderish(chars[right - 1]) {
        right -= 1;
    }

    while right > left && chars[right - 1].is_whitespace() {
        right -= 1;
    }

    if right <= left {
        return Some(String::new());
    }

    let content: String = chars[left..right].iter().collect();
    Some(content)
}

fn score_candidate(text: &str) -> i64 {
    let mut score: i64 = 0;
    for ch in text.chars() {
        if ch == '\u{FFFD}' {
            score -= 10; // replacement character indicates corruption
        } else if is_borderish(ch) {
            score -= 2;
        } else if ch.is_alphanumeric() {
            score += 4;
        } else if ch.is_ascii_punctuation() {
            score += 2;
        } else if ch.is_whitespace() {
            score += 1;
        } else {
            score -= 1;
        }
    }

    score
}

fn scrub_inline_borderish(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut in_border = false;

    for ch in text.chars() {
        if is_borderish(ch) {
            if !in_border && !result.is_empty() && !result.ends_with(' ') {
                result.push(' ');
            }
            in_border = true;
            continue;
        }

        if in_border && !result.is_empty() && !result.ends_with(' ') && !ch.is_whitespace() {
            result.push(' ');
        }

        in_border = false;
        result.push(ch);
    }

    result.trim_end().to_string()
}

fn push_content_line(
    output: &mut String,
    first: &mut bool,
    consecutive_empty: &mut usize,
    content: &str,
) {
    let trimmed = content.trim_end();
    if trimmed.trim().is_empty() {
        *consecutive_empty += 1;
        if *consecutive_empty > 2 {
            return;
        }
    } else {
        *consecutive_empty = 0;
    }

    if !*first {
        output.push('\n');
    }

    output.push_str(trimmed);
    *first = false;
}

fn strip_tui_lines(input: &str) -> String {
    let mut output = String::new();
    let mut first = true;
    let mut consecutive_empty = 0usize;

    for line in input.lines() {
        if is_mostly_borderish(line)
            || RE_BORDER_LINE.is_match(line)
            || RE_TITLED_BORDER.is_match(line)
        {
            continue;
        }

        if let Some(unwrapped) = unwrap_wrapped_line(line) {
            let cleaned_line = scrub_inline_borderish(&unwrapped);
            push_content_line(
                &mut output,
                &mut first,
                &mut consecutive_empty,
                &cleaned_line,
            );
            continue;
        }

        if let Some(caps) = RE_CONTENT_WRAPPER.captures(line) {
            if let Some(content) = caps.name("content") {
                let cleaned_line = scrub_inline_borderish(content.as_str());
                push_content_line(
                    &mut output,
                    &mut first,
                    &mut consecutive_empty,
                    &cleaned_line,
                );
                continue;
            }
        }

        // Fallback: retain the line as-is, but still enforce empty-line coalescing
        let cleaned_line = scrub_inline_borderish(line);
        push_content_line(
            &mut output,
            &mut first,
            &mut consecutive_empty,
            &cleaned_line,
        );
    }

    output.trim_end().to_string()
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
        let modified = self
            .modified
            .as_ref()
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
            eprintln!(
                "Warning: Cleaning reduced content by >90% ({} -> {} bytes)",
                self.original.len(),
                modified.len()
            );
            eprintln!("This might indicate over-aggressive cleaning.");
        }

        Ok(())
    }

    /// Commits the transaction by writing to clipboard with validation
    fn commit(self) -> Result<()> {
        let modified = self
            .modified
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
                    eprintln!(
                        "Verification failed: Clipboard content doesn't match expected result"
                    );
                    eprintln!(
                        "Expected {} bytes, got {} bytes",
                        expected_normalized.len(),
                        readback_normalized.len()
                    );
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
        // Try PowerShell first (WSL interop) with explicit UTF-8 encoding via Base64 transfer
        // This avoids all code page issues by transferring ASCII Base64 over the pipe.
        match Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "$b64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes(($OFS=\"`n\"; \"$(Get-Clipboard)\"))); Write-Output $b64"
            ])
            .output()
        {
            Ok(output) if output.status.success() => {
                let base64_str = String::from_utf8_lossy(&output.stdout).trim().to_string();

                // Decode Base64
                let decoded_bytes = BASE64_STANDARD.decode(&base64_str)
                    .context("Failed to decode Base64 from PowerShell")?;

                let text = String::from_utf8(decoded_bytes)
                    .context("Decoded Base64 is not valid UTF-8")?;

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
        // Use PowerShell with Base64 transfer for reliable encoding
        match Command::new("powershell.exe")
            .args([
                "-NoProfile",
                "-Command",
                "$b64 = $input | Out-String; if (-not [string]::IsNullOrWhiteSpace($b64)) { $bytes = [System.Convert]::FromBase64String($b64.Trim()); [System.Text.Encoding]::UTF8.GetString($bytes) | Set-Clipboard }"
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

                    // Encode to Base64 in Rust
                    let base64_str = BASE64_STANDARD.encode(data);

                    // Write Base64 string (safe ASCII)
                    stdin.write_all(base64_str.as_bytes())
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
                // powershell.exe not found - fallback logic
                // Try clip.exe (legacy, unreliable for utf-8 but better than nothing)
                 if data.is_ascii() {
                    eprintln!("Warning: powershell.exe not found, trying clip.exe...");
                    match Command::new("clip.exe").stdin(Stdio::piped()).spawn() {
                        Ok(mut child) => {
                            let mut stdin = child.stdin.take().unwrap();
                            stdin.write_all(data.as_bytes())?;
                            drop(stdin);
                            let status = child.wait()?;
                            if !status.success() { return Err(anyhow::anyhow!("clip.exe failed")); }
                            return Ok(());
                        }
                        Err(_) => {}
                    }
                }

                // Fall back to native clipboard (arboard)
                eprintln!("Warning: WSL detected but Windows interop not available.");
                let mut clipboard = arboard::Clipboard::new()?;
                clipboard.set_text(data)?;
                Ok(())
            }
            Err(e) => {
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
    let mut best = String::new();
    let mut best_score = i64::MIN;

    for variant in normalize_variants(input) {
        // First pass: strip ANSI escape codes (colors, cursor movement, etc.)
        let ansi_stripped = RE_ANSI.replace_all(&variant, "");

        // Second pass: remove TUI borders/content wrappers with heuristics
        let cleaned = strip_tui_lines(&ansi_stripped);

        let score = score_candidate(&cleaned);
        if score > best_score {
            best_score = score;
            best = cleaned;
        }
    }

    best.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_code_titled_border() {
        let input = "╭─── Claude Code v2.0.47 ──────────────────────────────────────────────────────────────────────────╮\n\
                     │                             │ Recent activity                                                    │\n\
                     │     Welcome back Ainesh!    │ No recent activity                                                 │\n\
                     │                             │ ────────────────────────────────────────────────────────────────── │\n\
                     │           ▐▛███▜▌           │ What's new                                                         │\n\
                     ╰──────────────────────────────────────────────────────────────────────────────────────────────────╯";

        // The expected output should have the top and bottom lines removed,
        // and the side borders removed from the content lines.

        let expected_contains = "Welcome back Ainesh!";
        let cleaned = clean_text(input);

        println!("Cleaned Output:\n{}", cleaned);

        assert!(
            cleaned.contains(expected_contains),
            "Should contain content"
        );
        assert!(
            !cleaned.contains("Claude Code v2.0.47"),
            "Should remove titled top border"
        );
        assert!(!cleaned.contains("╰───"), "Should remove bottom border");
        assert!(
            !cleaned.contains("│     Welcome"),
            "Should remove left border"
        );
    }

    #[test]
    fn test_ansi_stripping() {
        let input = "\x1b[31mHello\x1b[0m World";
        let cleaned = clean_text(input);
        assert_eq!(cleaned, "Hello World");

        let input_nested = "\x1b[1;31mBold Red\x1b[0m";
        let cleaned = clean_text(input_nested);
        assert_eq!(cleaned, "Bold Red");
    }

    #[test]
    fn test_code_with_pipes() {
        let input = "│ let x = a | b; │";
        let cleaned = clean_text(input);
        assert_eq!(cleaned, "let x = a | b;");
    }

    #[test]
    fn test_question_mark_preserved() {
        // '?' should be preserved as legitimate punctuation, not treated as borderish
        let input = "│ What's the status? │";
        let cleaned = clean_text(input);
        assert_eq!(cleaned, "What's the status?");
        
        // Multiple questions
        let input_multi = "│ How? Why? What? │";
        let cleaned_multi = clean_text(input_multi);
        assert_eq!(cleaned_multi, "How? Why? What?");
    }

    #[test]
    fn test_recovers_from_cp1252_mojibake() {
        let original =
            "╭─── Claude Code v2.0.47 ───╮\n│ Welcome back Ainesh! │\n╰───────────────────────╯";
        let corrupted = decode_windows_1252(original.as_bytes());

        // Ensure we actually produced mojibake
        assert_ne!(corrupted, original);

        let cleaned = clean_text(&corrupted);
        assert!(cleaned.contains("Welcome back Ainesh!"));
        assert!(!cleaned.contains("â"));
    }

    #[test]
    fn test_recovers_from_intersection_gibberish() {
        let gibberish = "?∩┐╜∩┐╜∩┐╜ Claude Code v2.0.47 ∩┐╜∩┐╜∩┐╜\n∩┐╜ Recent activity ∩┐╜\n∩┐╜ Welcome back Ainesh! ∩┐╜ No recent activity ∩┐╜\n∩┐╜ What's new ∩┐╜\n∩┐╜ /home/ain3sh ∩┐╜";

        let cleaned = clean_text(gibberish);

        // The important content lines should be preserved
        assert!(cleaned.contains("Welcome back Ainesh!"));
        assert!(cleaned.contains("No recent activity"));
        assert!(cleaned.contains("What's new"));
        
        // The mojibake border characters should be removed
        assert!(!cleaned.contains("∩┐╜"));
        
        // Note: The title line may be partially preserved because '?' is not
        // a border character (and shouldn't be, as it's legitimate punctuation).
        // This is acceptable as the primary goal is to extract the content lines.
    }
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
