use crate::issues::Hint;
use crate::issues::Issue;
use std::collections::BTreeSet;

pub struct LintPaxExtendedHeader {
    pub keywords: BTreeSet<String>,
    pub hints: BTreeSet<Hint>,
    pub issues: BTreeSet<Issue>,
    pub bytes: Vec<u8>,
}

impl LintPaxExtendedHeader {
    pub fn new(bytes: Vec<u8>) -> LintPaxExtendedHeader {
        let mut result = LintPaxExtendedHeader {
            keywords: BTreeSet::new(),
            hints: BTreeSet::new(),
            issues: BTreeSet::new(),
            bytes,
        };
        result.lint();
        result
    }

    fn lint_header(keywords: &mut BTreeSet<String>, vec: Vec<u8>) -> Option<Issue> {
        // check newline
        if vec[vec.len() - 1] != b'\n' {
            return Some(Issue::PaxHeaderNoNewline);
        }
        // check equal sign
        if let Some(p) = vec.iter().position(|&c| c == b'=') {
            let keyword = &vec[0..p];
            let _value = &vec[p + 1..vec.len()];
            // check keyword
            if keyword[0] == b' ' || keyword[0] == b'\t' {
                println!("{:?}", keyword);
                Some(Issue::PaxHeaderKeywordBlank)
            } else if let Ok(s) = String::from_utf8(keyword.to_vec()) {
                if keywords.contains(&s) {
                    Some(Issue::PaxHeaderKeywordDuplicate)
                } else if s.contains("size") {
                    Some(Issue::PaxSize)
                } else if s.contains("path") {
                    Some(Issue::PaxPath)
                } else {
                    keywords.insert(s);
                    None
                }
            } else {
                Some(Issue::PaxHeaderKeywordNoUtf8)
            }
        } else {
            Some(Issue::PaxHeaderNoEqualSign)
        }
    }

    fn lint(&mut self) {
        let mut start = 0;
        if self.bytes.is_empty() {
            self.issues.insert(Issue::PaxEmpty);
        }
        while start < self.bytes.len() && self.issues.is_empty() {
            // check blank
            if let Some(p) = self.bytes.iter().skip(start).position(|&c| c == b' ') {
                // check size
                let slice = &self.bytes[start..(start + p)];
                if let Ok(s) = String::from_utf8(slice.to_vec()) {
                    let issue = match s.parse::<u64>() {
                        Ok(n) => {
                            if s.starts_with("0") {
                                Some(Issue::PaxHeaderSizeOctal)
                            } else if n > i32::MAX as u64 {
                                Some(Issue::PaxHeaderSizeTooLarge)
                            } else {
                                let header_slice =
                                    &self.bytes[(start + s.len() + 1)..(start + n as usize)];
                                start += n as usize;
                                Self::lint_header(&mut self.keywords, header_slice.to_vec())
                            }
                        }
                        _ => Some(Issue::PaxHeaderSizeInvalid),
                    };
                    if let Some(i) = issue {
                        self.issues.insert(i);
                    }
                } else {
                    self.issues.insert(Issue::PaxHeaderSizeInvalid);
                }
            } else {
                self.issues.insert(Issue::PaxHeaderSizeInvalid);
            }
        }
    }
}
