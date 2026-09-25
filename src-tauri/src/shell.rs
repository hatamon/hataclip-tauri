use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(2);
const MAX_BYTES: usize = 100_000;
const MAX_LINES: usize = 1000;

#[derive(Debug, PartialEq, Eq)]
pub enum ShellError {
    Timeout,
    Failed(String),
    Empty,
}

pub fn command_error(err: &ShellError) -> String {
    match err {
        ShellError::Timeout => "時間切れ".into(),
        ShellError::Empty => "空".into(),
        ShellError::Failed(text) if text.trim().is_empty() => "コマンドに失敗した".into(),
        ShellError::Failed(text) => text.trim().to_string(),
    }
}

/// `{{sh: ...}}` と `:sh` と `#run` 本文の実行。失敗は貼らない。
pub fn run_script(script: &str) -> Result<String, ShellError> {
    run_script_with_stdin(script, None)
}

/// stdin に本文を流す（`:.!sh`）。`stdin` が `None` なら今までの `:sh` と同じ。
pub fn run_script_with_stdin(script: &str, stdin: Option<&str>) -> Result<String, ShellError> {
    let script = script.trim();
    if script.is_empty() {
        return Err(ShellError::Empty);
    }
    let mut child = spawn_shell(script, stdin.is_some()).map_err(|_| ShellError::Failed(String::new()))?;
    if let Some(input) = stdin {
        if let Some(mut pipe) = child.stdin.take() {
            use std::io::Write;
            let _ = pipe.write_all(input.as_bytes());
        }
    }
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(failed_stderr(&mut child));
                }
                break;
            }
            Ok(None) if started.elapsed() >= TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ShellError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => return Err(ShellError::Failed(String::new())),
        }
    }
    let mut buf = Vec::new();
    let Some(mut out) = child.stdout.take() else {
        return Err(ShellError::Failed(String::new()));
    };
    let _ = out.read_to_end(&mut buf);
    Ok(trim_output(&strip_ansi(&decode_output(&buf))))
}

fn failed_stderr(child: &mut std::process::Child) -> ShellError {
    let mut buf = Vec::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = err.read_to_end(&mut buf);
    }
    ShellError::Failed(trim_output(&strip_ansi(&decode_output(&buf))))
}

fn spawn_shell(script: &str, pipe_stdin: bool) -> std::io::Result<std::process::Child> {
    #[cfg(windows)]
    {
        for program in ["pwsh", "powershell"] {
            let mut command = Command::new(program);
            let utf8 = "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); $OutputEncoding = [Console]::OutputEncoding; ";
            let script = if program == "pwsh" {
                format!("{utf8}$PSStyle.OutputRendering='PlainText'; {script}")
            } else {
                format!("{utf8}{script}")
            };
            command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            command.stdin(if pipe_stdin {
                Stdio::piped()
            } else {
                Stdio::null()
            });
            command.env("NO_COLOR", "1");
            command.env("TERM", "dumb");
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.creation_flags(CREATE_NO_WINDOW);
            match command.spawn() {
                Ok(child) => return Ok(child),
                Err(_) => continue,
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "powershell",
        ))
    }
    #[cfg(not(windows))]
    {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        command.stdin(if pipe_stdin {
            Stdio::piped()
        } else {
            Stdio::null()
        });
        command.env("NO_COLOR", "1");
        command.env("TERM", "dumb");
        command.spawn()
    }
}

fn decode_output(buf: &[u8]) -> String {
    let buf = buf.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(buf);
    if let Ok(text) = std::str::from_utf8(buf) {
        return text.to_string();
    }
    decode_windows_acp(buf)
}

#[cfg(windows)]
fn decode_windows_acp(buf: &[u8]) -> String {
    use windows_sys::Win32::Globalization::{MultiByteToWideChar, CP_OEMCP};
    unsafe {
        let needed = MultiByteToWideChar(
            CP_OEMCP,
            0,
            buf.as_ptr(),
            buf.len() as i32,
            std::ptr::null_mut(),
            0,
        );
        if needed <= 0 {
            return String::from_utf8_lossy(buf).into_owned();
        }
        let mut wide = vec![0u16; needed as usize];
        let written = MultiByteToWideChar(
            CP_OEMCP,
            0,
            buf.as_ptr(),
            buf.len() as i32,
            wide.as_mut_ptr(),
            needed,
        );
        if written <= 0 {
            return String::from_utf8_lossy(buf).into_owned();
        }
        String::from_utf16_lossy(&wide[..written as usize])
    }
}

#[cfg(not(windows))]
fn decode_windows_acp(buf: &[u8]) -> String {
    String::from_utf8_lossy(buf).into_owned()
}

pub fn trim_output(text: &str) -> String {
    let text = text.replace("\r\n", "\n");
    let lines: Vec<&str> = text.lines().take(MAX_LINES).collect();
    let mut joined = lines.join("\n");
    if joined.len() > MAX_BYTES {
        joined.truncate(MAX_BYTES);
    }
    joined
        .strip_suffix('\n')
        .unwrap_or(&joined)
        .to_string()
}

fn strip_ansi(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] != '\u{1b}' {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        i += 1;
        let Some(next) = chars.get(i) else {
            break;
        };
        if *next == '[' {
            i += 1;
            while i < chars.len() {
                let ch = chars[i];
                i += 1;
                if ('@'..='~').contains(&ch) {
                    break;
                }
            }
            continue;
        }
        if *next == ']' {
            i += 1;
            while i < chars.len() {
                if chars[i] == '\u{7}' {
                    i += 1;
                    break;
                }
                if chars[i] == '\u{1b}' && chars.get(i + 1) == Some(&'\\') {
                    i += 2;
                    break;
                }
                i += 1;
            }
            continue;
        }
        i += 1;
    }
    out
}

/// `#run` なしなら `{{sh: ...}}` を空に。ありなら実行して埋め込む。
pub fn apply_sh(text: &str, run: bool) -> Result<String, ShellError> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
                if let Some(close) = crate::text::find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if let Some(cmd) = crate::text::arg_after(inner.trim(), "sh") {
                    if run {
                        out.push_str(&run_script(cmd)?);
                    }
                    i = close + 2;
                    continue;
                }
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    Ok(out)
}

pub fn has_sh_token(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
                if let Some(close) = crate::text::find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if crate::text::arg_after(inner.trim(), "sh").is_some() {
                    return true;
                }
                i = close + 2;
                continue;
            }
        }
        i += 1;
    }
    false
}

pub fn read_file_contents(path: &str) -> Result<String, ShellError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(ShellError::Failed(String::new()));
    }
    let data = std::fs::read(path).map_err(|_| ShellError::Failed(String::new()))?;
    let mut text = String::from_utf8_lossy(&data).into_owned();
    if text.len() > MAX_BYTES {
        text.truncate(MAX_BYTES);
    }
    Ok(trim_output(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_trailing_newline_and_caps_lines() {
        assert_eq!(trim_output("hello\n"), "hello");
        let many = (0..1200).map(|i| format!("{i}")).collect::<Vec<_>>().join("\n");
        assert_eq!(trim_output(&many).lines().count(), 1000);
    }

    #[cfg(not(windows))]
    #[test]
    fn run_script_reads_stdin() {
        let out = run_script_with_stdin("cat", Some("hello\n")).expect("cat");
        assert_eq!(out, "hello");
    }

    #[test]
    fn apply_sh_clears_tokens_without_run() {
        assert_eq!(apply_sh("a{{sh: echo hi}}b", false).unwrap(), "ab");
        assert_eq!(apply_sh("a{{sh echo hi}}b", false).unwrap(), "ab");
        assert!(has_sh_token("{{sh echo hi}}"));
        assert!(!has_sh_token("{{share}}"));
    }

    #[test]
    fn strips_ansi_color_codes() {
        assert_eq!(strip_ansi("\u{1b}[32;1mMode\u{1b}[0m"), "Mode");
        assert_eq!(strip_ansi("\u{1b}[44;1m.cargo\u{1b}[0m"), ".cargo");
        assert_eq!(
            trim_output(&strip_ansi("\u{1b}[32;1mhi\u{1b}[0m\n")),
            "hi"
        );
    }

    #[test]
    fn decodes_utf8_directory_header() {
        assert_eq!(
            decode_output("ディレクトリ:".as_bytes()),
            "ディレクトリ:"
        );
    }

    #[cfg(windows)]
    #[test]
    fn decodes_shift_jis_directory_header() {
        let bytes = [
            0x83, 0x66, 0x83, 0x42, 0x83, 0x8C, 0x83, 0x4E, 0x83, 0x67, 0x83, 0x8A, 0x3A,
        ];
        assert_eq!(decode_output(&bytes), "ディレクトリ:");
    }
}
