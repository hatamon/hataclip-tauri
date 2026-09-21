use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const TIMEOUT: Duration = Duration::from_secs(2);
const MAX_BYTES: usize = 100_000;
const MAX_LINES: usize = 1000;

#[derive(Debug, PartialEq, Eq)]
pub enum ShellError {
    Timeout,
    Failed,
    Empty,
}

/// `{{sh: ...}}` と `:sh` と `#run` 本文の実行。失敗は貼らない。
pub fn run_script(script: &str) -> Result<String, ShellError> {
    let script = script.trim();
    if script.is_empty() {
        return Err(ShellError::Empty);
    }
    let mut child = spawn_shell(script).map_err(|_| ShellError::Failed)?;
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err(ShellError::Failed);
                }
                break;
            }
            Ok(None) if started.elapsed() >= TIMEOUT => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(ShellError::Timeout);
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(20)),
            Err(_) => return Err(ShellError::Failed),
        }
    }
    let mut buf = Vec::new();
    let Some(mut out) = child.stdout.take() else {
        return Err(ShellError::Failed);
    };
    let _ = out.read_to_end(&mut buf);
    let text = String::from_utf8_lossy(&buf);
    Ok(trim_output(&text))
}

fn spawn_shell(script: &str) -> std::io::Result<std::process::Child> {
    #[cfg(windows)]
    {
        for program in ["pwsh", "powershell"] {
            let mut command = Command::new(program);
            command.args(["-NoProfile", "-NonInteractive", "-Command", script]);
            command.stdout(Stdio::piped()).stderr(Stdio::null());
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
        command.stdout(Stdio::piped()).stderr(Stdio::null());
        command.spawn()
    }
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

/// `#run` なしなら `{{sh: ...}}` を空に。ありなら実行して埋め込む。
pub fn apply_sh(text: &str, run: bool) -> Result<String, ShellError> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '{' && chars.get(i + 1) == Some(&'{') {
            if let Some(close) = find_close(&chars, i + 2) {
                let inner: String = chars[i + 2..close].iter().collect();
                if let Some(cmd) = inner.trim().strip_prefix("sh:") {
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

fn find_close(chars: &[char], start: usize) -> Option<usize> {
    let mut i = start;
    while i + 1 < chars.len() {
        if chars[i] == '}' && chars[i + 1] == '}' {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub fn has_sh_token(text: &str) -> bool {
    text.contains("{{sh:")
}

pub fn read_file_contents(path: &str) -> Result<String, ShellError> {
    let path = path.trim();
    if path.is_empty() {
        return Err(ShellError::Failed);
    }
    let data = std::fs::read(path).map_err(|_| ShellError::Failed)?;
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

    #[test]
    fn apply_sh_clears_tokens_without_run() {
        assert_eq!(apply_sh("a{{sh: echo hi}}b", false).unwrap(), "ab");
    }
}
