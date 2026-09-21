use std::collections::HashMap;

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
        raw.trim().parse().ok()
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
    }
}
