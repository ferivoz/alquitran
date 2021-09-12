use crate::issues::Hint;
use crate::issues::Issue;
use crate::lint::lint_nul_field;
use crate::lint::lint_number_field;
use crate::lint::lint_path_field;
use crate::lint::lint_string_field;
use crate::lint::LintResult;
use crate::lint::ERROR;
use crate::lint::WARNING;
use core::ops::Range;
use std::collections::BTreeSet;

pub const BLOCK_SIZE: usize = 512;

pub const NAME_RANGE: Range<usize> = 0..100;
pub const MODE_RANGE: Range<usize> = 100..108;
pub const UID_RANGE: Range<usize> = 108..116;
pub const GID_RANGE: Range<usize> = 116..124;
pub const SIZE_RANGE: Range<usize> = 124..136;
pub const MTIME_RANGE: Range<usize> = 136..148;
pub const CKSUM_RANGE: Range<usize> = 148..156;
pub const TYPEFLAG_RANGE: Range<usize> = 156..157;
pub const LINKNAME_RANGE: Range<usize> = 157..257;
pub const MAGIC_RANGE: Range<usize> = 257..263;
pub const VERSION_RANGE: Range<usize> = 263..265;
pub const UNAME_RANGE: Range<usize> = 265..297;
pub const GNAME_RANGE: Range<usize> = 297..329;
pub const DEVMAJOR_RANGE: Range<usize> = 329..337;
pub const DEVMINOR_RANGE: Range<usize> = 337..345;
pub const PREFIX_RANGE: Range<usize> = 345..500;
pub const USTAR_PADDING_RANGE: Range<usize> = 500..BLOCK_SIZE;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Format {
    Gnu,
    Pax,
    Ustar,
    V7,
}

pub struct LintHeader {
    pub hints: BTreeSet<Hint>,
    pub issues: BTreeSet<Issue>,
    pub bytes: [u8; BLOCK_SIZE],
    pub marks: [u8; BLOCK_SIZE],
    pub format: Format,
    pub linkname: String,
    pub mode: u64,
    pub path: String,
    pub size: u64,
    pub typeflag: u8,
}

impl LintHeader {
    pub fn new(bytes: [u8; BLOCK_SIZE]) -> LintHeader {
        let mut result = LintHeader {
            hints: BTreeSet::new(),
            issues: BTreeSet::new(),
            bytes,
            format: Format::V7,
            linkname: "".to_string(),
            marks: [0; BLOCK_SIZE],
            mode: 0,
            path: "".to_string(),
            size: 0,
            typeflag: b'0',
        };
        result.lint();
        result
    }

    pub fn get_data_block_count(&self) -> u64 {
        if self.typeflag == b'1' || self.typeflag == b'2' || self.typeflag == b'5' {
            return 0;
        }
        (self.size + 511) / BLOCK_SIZE as u64
    }

    fn insert<T>(&mut self, result: LintResult<T>, offset: usize) -> Option<T> {
        for &(hint, pos) in result.hints.iter() {
            self.hints.insert(hint);
            self.marks[offset + pos] |= WARNING;
        }
        for &(issue, pos) in result.issues.iter() {
            self.issues.insert(issue);
            self.marks[offset + pos] |= ERROR;
        }
        result.value
    }

    fn lint(&mut self) {
        let calculated = calculate_checksum(&self.bytes[..]);
        let cksum = self.lint_number(CKSUM_RANGE);
        let sum = match cksum {
            Some(n) => n,
            None => calculated + 1,
        };
        if sum != calculated {
            self.mark(CKSUM_RANGE, ERROR);
            self.issues.insert(Issue::InvalidChecksum);
            return;
        }

        let name = self.lint_path(NAME_RANGE);
        let mode = self.lint_number(MODE_RANGE);
        if let Some(n) = mode {
            if n > 0o7777 {
                self.mark(MODE_RANGE, ERROR);
                self.issues.insert(Issue::InvalidMode);
            }
            self.mode = n;
        }
        let _uid = self.lint_number(UID_RANGE);
        let _gid = self.lint_number(GID_RANGE);
        let size = self.lint_number(SIZE_RANGE);
        match size {
            Some(n) => {
                if n > 0x7FFFFFFF {
                    self.mark(SIZE_RANGE, ERROR);
                    self.issues.insert(Issue::LargeEntry);
                }
                self.size = n;
            }
            None => self.size = 0,
        }
        let _mtime = self.lint_number(MTIME_RANGE);
        self.typeflag = self.bytes[TYPEFLAG_RANGE.start];
        if (self.typeflag < b'0' || self.typeflag > b'7')
            && self.typeflag != b'g'
            && self.typeflag != b'x'
            && self.typeflag != 0
        {
            self.marks[TYPEFLAG_RANGE.start] |= ERROR;
            self.issues.insert(Issue::InvalidTypeFlag);
            self.typeflag = b'0';
        }

        if self.typeflag == 0 {
            self.typeflag = b'0';
        }

        let linkname = self.lint_path(LINKNAME_RANGE);

        /*
         * Alquitran prefers ustar format. As long as other formats can
         * be interpreted just like POSIX 2017 ustar archives there is
         * no good reason to complain.
         */
        if self.bytes[MAGIC_RANGE.start..VERSION_RANGE.end] == b"ustar\000"[..] {
            if self.typeflag == b'g' || self.typeflag == b'x' {
                self.format = Format::Pax;
            } else {
                self.format = Format::Ustar;
            }
        } else if self.bytes[MAGIC_RANGE.start..VERSION_RANGE.end] == b"ustar  \0"[..] {
            self.format = Format::Gnu;
        } else if self.bytes[MAGIC_RANGE.start..VERSION_RANGE.end] == [0; 8] {
            self.format = Format::V7;
        } else {
            if self.bytes[MAGIC_RANGE] != b"ustar\0"[..] {
                self.mark(MAGIC_RANGE, ERROR);
                self.issues.insert(Issue::InvalidMagic);
            }
            if self.bytes[VERSION_RANGE] != b"00"[..] {
                self.mark(VERSION_RANGE, ERROR);
                self.issues.insert(Issue::InvalidVersion);
            }
        }
        let _uname = self.lint_string(UNAME_RANGE);
        let _gname = self.lint_string(GNAME_RANGE);

        /*
         * The devmajor and devminor fields are number fields and
         * according to POSIX 2017 these fields are supposed to be
         * zero-leading. This is true for pax headers as well because
         * they are ustar headers, too. But too many implementations
         * do not set them. Accept this for now. At least a pax header
         * itself cannot be a char or block special file type.
         */
        if
        /*self.format == Format::Pax ||*/
        self.format == Format::Ustar {
            let devmajor = self.lint_number(DEVMAJOR_RANGE);
            if let Some(n) = devmajor {
                if n != 0 {
                    self.mark(DEVMAJOR_RANGE, ERROR);
                    self.issues.insert(Issue::DevMajorWithoutSpecialFile);
                }
            }
            let devminor = self.lint_number(DEVMINOR_RANGE);
            if let Some(n) = devminor {
                if n != 0 {
                    self.mark(DEVMINOR_RANGE, ERROR);
                    self.issues.insert(Issue::DevMinorWithoutSpecialFile);
                }
            }
        }
        let prefix = self.lint_path(PREFIX_RANGE);
        let _padding = self.lint_nul(USTAR_PADDING_RANGE);

        if name.is_none() && self.typeflag != b'5' {
            self.mark(NAME_RANGE, ERROR);
            self.issues.insert(Issue::EmptyName);
        }

        self.path = to_path(prefix, name);
        self.linkname = match linkname {
            Some(n) => String::from_utf8(n).unwrap(),
            None => "".to_string(),
        };

        self.lint_full_path();
        self.lint_linkname();
        self.lint_size();
    }

    fn lint_full_path(&mut self) {
        if self.path.is_empty() || self.path.starts_with('/') {
            self.mark(NAME_RANGE, ERROR);
            self.mark(PREFIX_RANGE, ERROR);
            if self.path.starts_with('/') {
                self.issues.insert(Issue::AbsolutePath);
            } else {
                self.issues.insert(Issue::EmptyPath);
            }
        }
        if self.path.contains("/../")
            || self.path.starts_with("../")
            || self.path.ends_with("/..")
            || self.path.contains("//")
        {
            self.mark(NAME_RANGE, ERROR);
            self.mark(PREFIX_RANGE, ERROR);
            if self.path.contains("//") {
                self.issues.insert(Issue::MultiSlashPath);
            } else {
                self.issues.insert(Issue::DotDotPath);
            }
        }
        if self.path.ends_with('/') && self.typeflag != b'5' {
            self.mark(NAME_RANGE, ERROR);
            self.mark(PREFIX_RANGE, ERROR);
            self.mark(TYPEFLAG_RANGE, ERROR);
            self.issues.insert(Issue::RegularDirectory);
        }
        if !self.path.ends_with('/') && self.typeflag == b'5' {
            self.mark(NAME_RANGE, ERROR);
            self.mark(PREFIX_RANGE, ERROR);
            self.mark(TYPEFLAG_RANGE, ERROR);
            self.issues.insert(Issue::DirectoryWithoutSlash);
        }
    }

    fn lint_linkname(&mut self) {
        if !self.linkname.is_empty() && self.typeflag != b'1' && self.typeflag != b'2' {
            self.mark(LINKNAME_RANGE, ERROR);
            self.marks[TYPEFLAG_RANGE.start] |= ERROR;
            self.issues.insert(Issue::LinknameForNonLink);
        }

        if self.linkname == self.path && self.typeflag == b'1' {
            self.mark(LINKNAME_RANGE, ERROR);
            self.mark(NAME_RANGE, ERROR);
            self.mark(PREFIX_RANGE, ERROR);
            self.marks[TYPEFLAG_RANGE.start] |= ERROR;
            self.issues.insert(Issue::LinkToItself);
        }
    }

    fn lint_nul(&mut self, range: Range<usize>) -> Option<bool> {
        let offset = range.start;
        let result = lint_nul_field(&self.bytes[range]);
        self.insert(result, offset)
    }

    fn lint_number(&mut self, range: Range<usize>) -> Option<u64> {
        let offset = range.start;
        let result = lint_number_field(&self.bytes[range]);
        self.insert(result, offset)
    }

    fn lint_path(&mut self, range: Range<usize>) -> Option<Vec<u8>> {
        let offset = range.start;
        let result = lint_path_field(&self.bytes[range]);
        self.insert(result, offset)
    }

    fn lint_size(&mut self) {
        let no_data = self.typeflag == b'1' || self.typeflag == b'2' || self.typeflag == b'5';
        if no_data && self.size != 0 {
            self.mark(SIZE_RANGE, ERROR);
            self.marks[TYPEFLAG_RANGE.start] |= ERROR;
            self.issues.insert(Issue::NoDataWithSize);
        }
    }

    fn lint_string(&mut self, range: Range<usize>) -> Option<Vec<u8>> {
        let offset = range.start;
        let result = lint_string_field(&self.bytes[range]);
        self.insert(result, offset)
    }

    fn mark(&mut self, range: Range<usize>, b: u8) {
        for n in range {
            self.marks[n] |= b;
        }
    }
}

fn calculate_checksum(bytes: &[u8]) -> u64 {
    let empty: [u8; 8] = [b' '; 8];
    bytes[0..148]
        .iter()
        .chain(&empty[..])
        .chain(&bytes[156..BLOCK_SIZE])
        .fold(0, |a, b| a + (*b as u64))
}

fn normalize(path: String) -> String {
    let mut simple = path.clone();
    while simple.contains("//") {
        simple = simple.replace("//", "/");
    }
    let reduced = String::from(
        simple
            .trim_start_matches("./")
            .replace("/./", "/")
            .trim_end_matches('/')
            .trim_end_matches("/."),
    );
    let mut normalized;
    if reduced.is_empty() && !path.is_empty() {
        normalized = String::from(&path[0..1]);
    } else {
        normalized = reduced;
    }
    if !normalized.starts_with('/') && path.ends_with('/') {
        normalized.push('/');
    }
    normalized
}

fn to_path(prefix: Option<Vec<u8>>, name: Option<Vec<u8>>) -> String {
    let path;
    let prefix_string = match prefix {
        Some(n) => String::from_utf8(n).unwrap(),
        None => "".to_string(),
    };
    let name_string = match name {
        Some(n) => String::from_utf8(n).unwrap(),
        None => "".to_string(),
    };
    if prefix_string.is_empty() {
        path = name_string;
    } else if name_string.is_empty() {
        path = prefix_string;
    } else {
        path = prefix_string + "/" + &name_string;
    }
    normalize(path)
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use super::to_path;

    #[test]
    fn test_normalize() {
        assert_eq!("test", normalize("./test".to_string()));
        assert_eq!("test/", normalize(".///test/./".to_string()));
        assert_eq!("test", normalize("././test/.".to_string()));
        assert_eq!("/test", normalize("////////test".to_string()));
        assert_eq!(".", normalize(".".to_string()));
        assert_eq!("./", normalize("./".to_string()));
        assert_eq!("./", normalize(".//".to_string()));
    }

    #[test]
    fn test_to_path() {
        assert_eq!("", to_path(None, None));
        assert_eq!("", to_path(Some(b"".to_vec()), Some(b"".to_vec())));
        assert_eq!("file", to_path(Some(b"".to_vec()), Some(b"file".to_vec())));
        assert_eq!(
            "directory/file",
            to_path(Some(b"directory".to_vec()), Some(b"file".to_vec()))
        );
        assert_eq!("directory", to_path(Some(b"directory".to_vec()), None));
    }
}
