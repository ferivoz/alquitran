use crate::header::BLOCK_SIZE;
use crate::header::Format;
use crate::header::LintHeader;
use crate::issues::Hint;
use crate::issues::Issue;
use crate::lint::ERROR;
use crate::lint::lint_nul_field;
use crate::pax::LintPaxExtendedHeader;
use std::collections::BTreeSet;
use std::io::Read;
use std::io::Result;

pub struct Dump {
    pub bytes: [u8; BLOCK_SIZE],
    pub marks: [u8; BLOCK_SIZE],
    pub offset: usize,
}

pub struct ArchiveLintResult {
    pub dump: Option<Dump>,
    pub duplicated_paths: BTreeSet<String>,
    pub format: Option<Format>,
    pub hints: BTreeSet<Hint>,
    pub issues: BTreeSet<Issue>,
}

impl ArchiveLintResult {
    pub fn is_portable(&self) -> bool {
        self.issues.is_empty() && self.duplicated_paths.is_empty() && self.dump.is_none()
    }

    fn insert(&mut self, header: LintHeader, offset: usize) {
        self.dump = Some(Dump {
            bytes: header.bytes,
            marks: header.marks,
            offset,
        });
        for &hint in header.hints.iter() {
            self.hints.insert(hint);
        }
        for &issue in header.issues.iter() {
            self.issues.insert(issue);
        }
    }
}

pub struct Archive {
    reader: Box<dyn Read>,
}

impl Archive {
    pub fn new(reader: Box<dyn Read>) -> Archive {
        Archive { reader }
    }

    pub fn lint(&mut self) -> Result<ArchiveLintResult> {
        let mut result = ArchiveLintResult {
            dump: None,
            duplicated_paths: BTreeSet::new(),
            format: None,
            hints: BTreeSet::new(),
            issues: BTreeSet::new(),
        };
        let mut eoa = 0;
        let mut i = 0;
        let mut paths = BTreeSet::new();
        let mut read_only_directories = BTreeSet::<String>::new();
        let mut links = BTreeSet::<String>::new();

        loop {
            let mut block: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
            self.reader.read_exact(&mut block[..])?;
            if lint_nul_field(&block).value.unwrap() {
                eoa += 1;
                if eoa == 2 {
                    break;
                }
                i += 1;
                continue;
            }
            if eoa != 0 {
                let empty: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
                let header = LintHeader::new(empty);
                result.insert(header, i - 1);
                return Ok(result);
            }
            let header = LintHeader::new(block);
            if result.format.is_none() {
                result.format = Some(header.format);
            } else {
                let archive_format = result.format.unwrap();
                if archive_format == Format::Pax {
                    if header.format != Format::Pax && header.format != Format::Ustar {
                        result.insert(header, i);
                        result.issues.insert(Issue::FormatChanged);
                        return Ok(result);
                    }
                    result.format = Some(Format::Pax);
                } else if archive_format == Format::Ustar {
                    if header.format != Format::Pax && header.format != Format::Ustar {
                        result.insert(header, i);
                        result.issues.insert(Issue::FormatChanged);
                        return Ok(result);
                    }
                    result.format = Some(header.format);
                } else if header.format != archive_format {
                    result.insert(header, i);
                    result.issues.insert(Issue::FormatChanged);
                    return Ok(result);
                } else {
                    result.format = Some(header.format);
                }
            }
            if !header.issues.is_empty() {
                result.insert(header, i);
                return Ok(result);
            } else {
                let header_offset = i;
                let mut data: [u8; BLOCK_SIZE] = [0; BLOCK_SIZE];
                let count = header.get_data_block_count();
                if count > 0 {
                    let copy = header.typeflag == b'x' || header.typeflag == b'g';
                    let mut xheader = Vec::new();
                    for _b in 0..(count - 1) {
                        self.reader.read_exact(&mut data[..])?;
                        if copy {
                            xheader.append(&mut data.to_vec());
                        }
                        i += 1;
                    }
                    self.reader.read_exact(&mut data[..])?;
                    let offset: usize = (header.size % BLOCK_SIZE as u64) as usize;
                    if copy {
                        xheader.append(&mut data[0..offset].to_vec());
                        let pheader = LintPaxExtendedHeader::new(xheader.clone());
                        if let Some(i) = pheader.issues.into_iter().next() {
                            result.insert(header, header_offset);
                            result.issues.insert(i);
                            return Ok(result);
                        }
                    }
                    i += 1;
                    if offset != 0 && data[offset..BLOCK_SIZE].iter().any(|&x| x != 0) {
                        let mut dump = Dump {
                            bytes: data,
                            marks: [0; BLOCK_SIZE],
                            offset: i,
                        };
                        for n in offset..BLOCK_SIZE {
                            if dump.bytes[n] != 0 {
                                dump.marks[n] |= ERROR;
                            }
                        }
                        result.dump = Some(dump);
                        result.issues.insert(Issue::DataPaddingNotNul);
                        return Ok(result);
                    }
                }
            }
            let mut path = header.path.clone();
            if path.ends_with('/') {
                path.pop();
            }
            for dir in read_only_directories.iter() {
                if path.starts_with(dir.as_str()) {
                    result.insert(header, i);
                    result.issues.insert(Issue::ReadOnlyDirectoryWithEntries);
                    return Ok(result);
                }
            }
            for link in links.iter() {
                if path.starts_with(link.as_str()) {
                    result.insert(header, i);
                    result.issues.insert(Issue::LinkIsParent);
                    return Ok(result);
                }
            }
            if header.typeflag == b'5' && (header.mode & 0o200) == 0 {
                let mut dir = path.clone();
                dir.push('/');
                read_only_directories.insert(dir);
            }
            if header.typeflag == b'1' || header.typeflag == b'2' {
                let mut dir = path.clone();
                dir.push('/');
                links.insert(dir);
            }
            if paths.contains(&path) {
                result.insert(header, i);
                result.duplicated_paths.insert(path);
                return Ok(result);
            } else {
                paths.insert(path);
            }
            i += 1;
        }
        let mut eof = Vec::new();
        self.reader.read_to_end(&mut eof)?;
        if eof.iter().any(|&b| b != 0) {
            result.issues.insert(Issue::TrailingByteNotNul);
        }
        Ok(result)
    }
}
