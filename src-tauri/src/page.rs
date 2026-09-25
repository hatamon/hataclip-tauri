use std::io::{self, Write};

pub fn present(text: &str) -> io::Result<()> {
    let lines = split_lines(text);
    let tty = io::IsTerminal::is_terminal(&io::stdout());
    let height = if tty { console_rows() } else { None };
    match height {
        Some(height) if lines.len() > height && height > 0 => scroll(&lines, height),
        _ => {
            let mut out = io::stdout().lock();
            write_text(&mut out, text)
        }
    }
}

pub fn next_offset(offset: usize, lines: usize, height: usize, down: bool) -> usize {
    if height == 0 || lines <= height {
        return 0;
    }
    let max = lines - height;
    if down {
        offset.saturating_add(1).min(max)
    } else {
        offset.saturating_sub(1)
    }
}

fn split_lines(text: &str) -> Vec<&str> {
    let mut lines: Vec<&str> = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect();
    if lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    if lines.is_empty() {
        lines.push("");
    }
    lines
}

fn write_text(out: &mut impl Write, text: &str) -> io::Result<()> {
    out.write_all(text.as_bytes())?;
    if !text.ends_with('\n') {
        out.write_all(b"\n")?;
    }
    out.flush()
}

fn scroll(lines: &[&str], height: usize) -> io::Result<()> {
    let mut raw = RawTerm::enter()?;
    let mut offset = 0;
    loop {
        draw(lines, offset, height)?;
        match raw.read_key()? {
            Key::Down => offset = next_offset(offset, lines.len(), height, true),
            Key::Up => offset = next_offset(offset, lines.len(), height, false),
            Key::Quit => break,
        }
    }
    Ok(())
}

fn draw(lines: &[&str], offset: usize, height: usize) -> io::Result<()> {
    let mut out = io::stdout().lock();
    write!(out, "\x1b[?1049h\x1b[H")?;
    for line in lines.iter().skip(offset).take(height) {
        writeln!(out, "{line}\x1b[K")?;
    }
    write!(out, "\x1b[J")?;
    out.flush()
}

enum Key {
    Down,
    Up,
    Quit,
}

struct RawTerm {
    #[cfg(windows)]
    input: std::fs::File,
    #[cfg(windows)]
    mode: u32,
    #[cfg(unix)]
    tty: std::fs::File,
    #[cfg(unix)]
    term: libc::termios,
}

impl Drop for RawTerm {
    fn drop(&mut self) {
        self.restore();
        let mut out = io::stdout().lock();
        let _ = write!(out, "\x1b[?1049l");
        let _ = out.flush();
    }
}

#[cfg(windows)]
fn console_rows() -> Option<usize> {
    use windows_sys::Win32::System::Console::{
        GetConsoleScreenBufferInfo, GetStdHandle, CONSOLE_SCREEN_BUFFER_INFO, STD_OUTPUT_HANDLE,
    };
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut info = std::mem::zeroed::<CONSOLE_SCREEN_BUFFER_INFO>();
        if GetConsoleScreenBufferInfo(handle, &mut info) == 0 {
            return None;
        }
        Some((info.srWindow.Bottom - info.srWindow.Top + 1) as usize)
    }
}

#[cfg(windows)]
impl RawTerm {
    fn enter() -> io::Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Console::{
            GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT,
            ENABLE_VIRTUAL_TERMINAL_PROCESSING, STD_OUTPUT_HANDLE,
        };
        unsafe {
            let output = GetStdHandle(STD_OUTPUT_HANDLE);
            let mut out_mode = 0;
            if GetConsoleMode(output, &mut out_mode) != 0 {
                SetConsoleMode(output, out_mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
            }
        }
        let input = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("CONIN$")?;
        let handle = input.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        let mut mode = 0;
        unsafe {
            if GetConsoleMode(handle, &mut mode) == 0 {
                return Err(io::Error::last_os_error());
            }
            let next = mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT);
            if SetConsoleMode(handle, next) == 0 {
                return Err(io::Error::last_os_error());
            }
        }
        Ok(Self { input, mode })
    }

    fn restore(&mut self) {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Console::SetConsoleMode;
        let handle = self.input.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        unsafe {
            SetConsoleMode(handle, self.mode);
        }
    }

    fn read_key(&mut self) -> io::Result<Key> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::Console::{ReadConsoleInputW, INPUT_RECORD, KEY_EVENT};
        let handle = self.input.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
        loop {
            let mut record = unsafe { std::mem::zeroed::<INPUT_RECORD>() };
            let mut read = 0;
            let ok = unsafe { ReadConsoleInputW(handle, &mut record, 1, &mut read) };
            if ok == 0 {
                return Err(io::Error::last_os_error());
            }
            if read == 0 || record.EventType != KEY_EVENT as u16 {
                continue;
            }
            let key = unsafe { record.Event.KeyEvent };
            if key.bKeyDown == 0 {
                continue;
            }
            let ch = unsafe { key.uChar.UnicodeChar };
            if key.wVirtualKeyCode == 0x1B || ch == 'q' as u16 || ch == 'Q' as u16 {
                return Ok(Key::Quit);
            }
            if ch == 'j' as u16 || ch == 'J' as u16 {
                return Ok(Key::Down);
            }
            if ch == 'k' as u16 || ch == 'K' as u16 {
                return Ok(Key::Up);
            }
        }
    }
}

#[cfg(unix)]
fn console_rows() -> Option<usize> {
    let tty = std::fs::File::open("/dev/tty").ok()?;
    use std::os::fd::AsRawFd;
    let mut size = libc::winsize {
        ws_row: 0,
        ws_col: 0,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let ok = unsafe { libc::ioctl(tty.as_raw_fd(), libc::TIOCGWINSZ, &mut size) };
    if ok == -1 || size.ws_row == 0 {
        None
    } else {
        Some(size.ws_row as usize)
    }
}

#[cfg(unix)]
impl RawTerm {
    fn enter() -> io::Result<Self> {
        let tty = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;
        use std::os::fd::AsRawFd;
        let fd = tty.as_raw_fd();
        let mut term = unsafe { std::mem::zeroed::<libc::termios>() };
        if unsafe { libc::tcgetattr(fd, &mut term) } == -1 {
            return Err(io::Error::last_os_error());
        }
        let mut raw = term;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } == -1 {
            return Err(io::Error::last_os_error());
        }
        Ok(Self { tty, term })
    }

    fn restore(&mut self) {
        use std::os::fd::AsRawFd;
        unsafe {
            libc::tcsetattr(self.tty.as_raw_fd(), libc::TCSANOW, &self.term);
        }
    }

    fn read_key(&mut self) -> io::Result<Key> {
        use std::io::Read;
        let mut buf = [0u8; 1];
        loop {
            self.tty.read_exact(&mut buf)?;
            match buf[0] {
                b'j' | b'J' => return Ok(Key::Down),
                b'k' | b'K' => return Ok(Key::Up),
                b'q' | b'Q' | 0x1b => return Ok(Key::Quit),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_stops_at_the_ends() {
        assert_eq!(next_offset(0, 10, 4, false), 0);
        assert_eq!(next_offset(0, 10, 4, true), 1);
        assert_eq!(next_offset(6, 10, 4, true), 6);
        assert_eq!(next_offset(6, 10, 4, false), 5);
        assert_eq!(next_offset(3, 4, 10, true), 0);
    }

    #[test]
    fn carriage_return_is_not_part_of_the_line() {
        assert_eq!(split_lines("a\r\nb\r\n"), vec!["a", "b"]);
    }
}
