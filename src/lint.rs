use crate::issues::Hint;
use crate::issues::Issue;
use std::str;

pub const WARNING: u8 = 1;
pub const ERROR: u8 = 2;

pub struct LintResult<V> {
    pub value: Option<V>,
    pub hints: Vec<(Hint, usize)>,
    pub issues: Vec<(Issue, usize)>,
}

pub fn lint_nul_field(bytes: &[u8]) -> LintResult<bool> {
    let mut issues = Vec::new();
    for (n, &b) in bytes.iter().enumerate() {
        if b != 0 {
            issues.push((Issue::UnusedByteNotNul, n));
        }
    }
    LintResult {
        value: Some(issues.is_empty()),
        hints: Vec::new(),
        issues,
    }
}

pub fn lint_number_field(bytes: &[u8]) -> LintResult<u64> {
    let mut num = true;
    let mut eos = false;
    let mut chars = Vec::new();
    let mut issues = Vec::new();
    for (n, &b) in bytes.iter().enumerate() {
        if eos {
            if b != 0 && b != b' ' {
                issues.push((Issue::UnusedByteNotNul, n));
            }
        } else if is_octal_digit(b) {
            chars.push(b);
        } else if b == 0 || b == b' ' {
            eos = true;
        } else {
            issues.push((Issue::InvalidNumber, n));
            num = false;
        }
    }

    let value;
    if !num {
        value = None;
    } else if chars.is_empty() {
        issues.push((Issue::NoNumber, 0));
        value = None;
    } else if !eos {
        issues.push((Issue::UnterminatedNumber, bytes.len() - 1));
        value = None;
    } else {
        let number = u64::from_str_radix(str::from_utf8(&chars).unwrap(), 8).unwrap();
        value = Some(number);
    }
    LintResult {
        value,
        hints: Vec::new(),
        issues,
    }
}

pub fn lint_path_field(bytes: &[u8]) -> LintResult<Vec<u8>> {
    let mut eos = false;
    let mut chars = Vec::new();
    let mut hints = Vec::new();
    let mut issues = Vec::new();
    for (n, &b) in bytes.iter().enumerate() {
        if eos {
            if b != 0 {
                issues.push((Issue::UnusedByteNotNul, n));
            }
        } else if b == 0 {
            eos = true;
        } else {
            if !is_portable_char(b) && b != b'/' {
                hints.push((Hint::UnportableCharInPath, n));
            }
            chars.push(b);
        }
    }
    LintResult {
        value: Some(chars),
        hints,
        issues,
    }
}

pub fn lint_string_field(bytes: &[u8]) -> LintResult<Vec<u8>> {
    let mut eos = false;
    let mut chars = Vec::new();
    let mut hints = Vec::new();
    let mut issues = Vec::new();
    for (n, &b) in bytes.iter().enumerate() {
        if eos {
            if b != 0 {
                issues.push((Issue::UnusedByteNotNul, n));
            }
        } else if b == 0 {
            eos = true;
        } else {
            if !is_portable_char(b) {
                hints.push((Hint::UnportableCharInString, n));
            }
            chars.push(b);
        }
    }
    let value = if !eos {
        if !bytes.is_empty() {
            issues.push((Issue::UnterminatedString, bytes.len() - 1));
        }
        None
    } else {
        Some(chars)
    };
    LintResult {
        value,
        hints,
        issues,
    }
}

fn is_octal_digit(c: u8) -> bool {
    (b'0'..=b'7').contains(&c)
}

fn is_portable_char(c: u8) -> bool {
    c.is_ascii_uppercase()
        || c.is_ascii_lowercase()
        || c.is_ascii_digit()
        || c == b'.'
        || c == b'_'
        || c == b'-'
}
