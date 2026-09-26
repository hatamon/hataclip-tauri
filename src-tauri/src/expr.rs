use std::collections::HashMap;

/// `0b` `0o` `0x` と10進の整数。負・小数・空は読まない。
pub fn parse_unsigned(raw: &str) -> Option<u64> {
    let raw = raw.trim();
    if raw.is_empty() || raw.starts_with('+') || raw.starts_with('-') {
        return None;
    }
    let (base, digits) = if let Some(rest) = strip_radix_prefix(raw, "0b") {
        (2u32, rest)
    } else if let Some(rest) = strip_radix_prefix(raw, "0o") {
        (8, rest)
    } else if let Some(rest) = strip_radix_prefix(raw, "0x") {
        (16, rest)
    } else if raw.bytes().all(|byte| byte.is_ascii_digit()) {
        (10, raw)
    } else {
        return None;
    };
    if digits.is_empty() {
        return None;
    }
    u64::from_str_radix(digits, base).ok()
}

/// 流れの各行をその進数にする。1行でも読めなければ `None`。
pub fn convert_base(text: &str, kind: &str) -> Option<String> {
    if text.is_empty() {
        return None;
    }
    let trailing = text.ends_with('\n');
    let mut out = Vec::new();
    for line in text.lines() {
        let value = parse_unsigned(line)?;
        out.push(format_base(value, kind)?);
    }
    if out.is_empty() {
        return None;
    }
    let mut joined = out.join("\n");
    if trailing {
        joined.push('\n');
    }
    Some(joined)
}

fn format_base(value: u64, kind: &str) -> Option<String> {
    Some(match kind {
        "bin" => format!("0b{value:b}"),
        "oct" => format!("0o{value:o}"),
        "dec" => format!("{value}"),
        "hex" => format!("0x{value:x}"),
        _ => return None,
    })
}

fn strip_radix_prefix<'a>(raw: &'a str, prefix: &str) -> Option<&'a str> {
    let bytes = raw.as_bytes();
    if bytes.len() < prefix.len() || !bytes[..prefix.len()].eq_ignore_ascii_case(prefix.as_bytes()) {
        return None;
    }
    Some(&raw[prefix.len()..])
}

fn u64_as_echo(value: u64) -> Option<f64> {
    let number = value as f64;
    if number as u64 == value {
        Some(number)
    } else {
        None
    }
}

fn echo_number(raw: &str) -> Option<f64> {
    let raw = raw.trim();
    if strip_radix_prefix(raw, "0b").is_some()
        || strip_radix_prefix(raw, "0o").is_some()
        || strip_radix_prefix(raw, "0x").is_some()
    {
        return parse_unsigned(raw).and_then(u64_as_echo);
    }
    raw.parse().ok()
}

/// `2+3` `*` `/` が先。`()` で変えられる。変数は数値として読む。
pub fn eval_with(input: &str, vars: &HashMap<String, String>) -> Option<f64> {
    let chars: Vec<char> = input.chars().collect();
    let mut parser = Parser {
        chars,
        i: 0,
        vars,
    };
    let value = parser.expr()?;
    parser.skip();
    if parser.i != parser.chars.len() || !value.is_finite() {
        return None;
    }
    Some(value)
}

pub fn format_number(n: f64) -> String {
    if n.abs() < 1e15 && (n.round() - n).abs() < 1e-10 {
        format!("{}", n.round() as i64)
    } else {
        let text = format!("{n}");
        if let Some(trimmed) = text.strip_suffix(".0") {
            trimmed.to_string()
        } else {
            text
        }
    }
}

struct Parser<'a> {
    chars: Vec<char>,
    i: usize,
    vars: &'a HashMap<String, String>,
}

impl Parser<'_> {
    fn expr(&mut self) -> Option<f64> {
        let mut value = self.term()?;
        loop {
            self.skip();
            match self.peek() {
                Some('+') => {
                    self.i += 1;
                    value += self.term()?;
                }
                Some('-') => {
                    self.i += 1;
                    value -= self.term()?;
                }
                _ => return Some(value),
            }
        }
    }

    fn term(&mut self) -> Option<f64> {
        let mut value = self.unary()?;
        loop {
            self.skip();
            match self.peek() {
                Some('*') => {
                    self.i += 1;
                    value *= self.unary()?;
                }
                Some('/') => {
                    self.i += 1;
                    let denom = self.unary()?;
                    if denom == 0.0 {
                        return None;
                    }
                    value /= denom;
                }
                _ => return Some(value),
            }
        }
    }

    fn unary(&mut self) -> Option<f64> {
        self.skip();
        if self.peek() == Some('-') {
            self.i += 1;
            return Some(-self.unary()?);
        }
        if self.peek() == Some('+') {
            self.i += 1;
            return self.unary();
        }
        self.primary()
    }

    fn primary(&mut self) -> Option<f64> {
        self.skip();
        if self.peek() == Some('(') {
            self.i += 1;
            let value = self.expr()?;
            self.skip();
            if self.peek() != Some(')') {
                return None;
            }
            self.i += 1;
            return Some(value);
        }
        if self.peek().map_or(false, |ch| ch.is_ascii_digit() || ch == '.') {
            if self.peek() == Some('0') {
                if let Some(mark) = self.chars.get(self.i + 1).copied() {
                    if matches!(mark.to_ascii_lowercase(), 'b' | 'o' | 'x') {
                        return self.radix();
                    }
                }
            }
            return self.number();
        }
        if self
            .peek()
            .map_or(false, |ch| ch.is_ascii_alphabetic() || ch == '_')
        {
            return self.ident();
        }
        None
    }

    fn number(&mut self) -> Option<f64> {
        let start = self.i;
        let mut seen_dot = false;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.i += 1;
            } else if ch == '.' && !seen_dot {
                seen_dot = true;
                self.i += 1;
            } else {
                break;
            }
        }
        let token: String = self.chars[start..self.i].iter().collect();
        if token.is_empty() || token == "." {
            return None;
        }
        token.parse().ok()
    }

    fn radix(&mut self) -> Option<f64> {
        let mark = self.chars.get(self.i + 1).copied()?.to_ascii_lowercase();
        let base = match mark {
            'b' => 2,
            'o' => 8,
            'x' => 16,
            _ => return None,
        };
        let start = self.i + 2;
        let mut end = start;
        while self
            .chars
            .get(end)
            .map_or(false, |ch| ch.to_digit(base).is_some())
        {
            end += 1;
        }
        if end == start {
            return None;
        }
        let token: String = self.chars[start..end].iter().collect();
        let value = u64::from_str_radix(&token, base).ok().and_then(u64_as_echo)?;
        self.i = end;
        Some(value)
    }

    fn ident(&mut self) -> Option<f64> {
        let start = self.i;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                self.i += 1;
            } else {
                break;
            }
        }
        let name: String = self.chars[start..self.i].iter().collect();
        let raw = self.vars.get(&name)?;
        echo_number(raw)
    }

    fn skip(&mut self) {
        while self.peek().map_or(false, |ch| ch.is_whitespace()) {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn eval(input: &str) -> Option<f64> {
        eval_with(input, &HashMap::new())
    }

    #[test]
    fn precedence_and_parens() {
        assert_eq!(eval("2+3").map(format_number).as_deref(), Some("5"));
        assert_eq!(eval("2+3*4").map(format_number).as_deref(), Some("14"));
        assert_eq!(eval("(2+3)*4").map(format_number).as_deref(), Some("20"));
        assert_eq!(eval("10/2").map(format_number).as_deref(), Some("5"));
        assert_eq!(eval("1/2").map(format_number).as_deref(), Some("0.5"));
        assert_eq!(eval("2*-3").map(format_number).as_deref(), Some("-6"));
        assert_eq!(eval("-(2+3)").map(format_number).as_deref(), Some("-5"));
        assert_eq!(eval(" 2 + 3 ").map(format_number).as_deref(), Some("5"));
    }

    #[test]
    fn variables_and_errors() {
        let mut vars = HashMap::new();
        vars.insert("a".into(), "2".into());
        assert_eq!(
            eval_with("a+3", &vars).map(format_number).as_deref(),
            Some("5")
        );
        assert_eq!(eval("1/0"), None);
        assert_eq!(eval("(2+3"), None);
        assert_eq!(eval("2+"), None);
        assert_eq!(eval("nope"), None);
        vars.insert("a".into(), "0x10".into());
        assert_eq!(
            eval_with("a+1", &vars).map(format_number).as_deref(),
            Some("17")
        );
    }

    #[test]
    fn reads_binary_octal_and_hex_in_the_expression() {
        assert_eq!(eval("0x10 + 0b1").map(format_number).as_deref(), Some("17"));
        assert_eq!(eval("0o10").map(format_number).as_deref(), Some("8"));
        assert_eq!(eval("0XFF").map(format_number).as_deref(), Some("255"));
        assert_eq!(eval("0x"), None);
        assert_eq!(eval("0b2"), None);
        assert_eq!(eval("1/2").map(format_number).as_deref(), Some("0.5"));
    }

    #[test]
    fn converts_between_bases_and_rejects_the_rest() {
        assert_eq!(convert_base("255", "hex").as_deref(), Some("0xff"));
        assert_eq!(convert_base("0xff", "bin").as_deref(), Some("0b11111111"));
        assert_eq!(convert_base("0b1010", "oct").as_deref(), Some("0o12"));
        assert_eq!(convert_base("0xff", "dec").as_deref(), Some("255"));
        assert_eq!(convert_base("0", "hex").as_deref(), Some("0x0"));
        assert_eq!(convert_base("10\n0x10", "hex").as_deref(), Some("0xa\n0x10"));
        assert_eq!(convert_base("10\nnope", "hex"), None);
        assert_eq!(convert_base("-1", "hex"), None);
        assert_eq!(convert_base("0.5", "hex"), None);
        assert_eq!(convert_base("", "hex"), None);
        assert_eq!(convert_base("255\n", "bin").as_deref(), Some("0b11111111\n"));
    }
}
