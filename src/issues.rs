use std::collections::BTreeSet;

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Hint {
    UnportableCharInPath,
    UnportableCharInString,
}

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Issue {
    AbsolutePath,
    DataPaddingNotNul,
    DevMajorWithoutSpecialFile,
    DevMinorWithoutSpecialFile,
    DirectoryWithoutSlash,
    DotDotPath,
    EmptyName,
    EmptyPath,
    FormatChanged,
    InvalidChecksum,
    InvalidMagic,
    InvalidMode,
    InvalidNumber,
    InvalidTypeFlag,
    InvalidVersion,
    LargeEntry,
    LinkToItself,
    LinkIsParent,
    LinknameForNonLink,
    MultiSlashPath,
    NoDataWithSize,
    NoNumber,
    PaxPath,
    PaxSize,
    ReadOnlyDirectoryWithEntries,
    RegularDirectory,
    TrailingByteNotNul,
    UnterminatedNumber,
    UnterminatedString,
    UnusedByteNotNul,
}

pub fn eprint_issues(issues: &BTreeSet<Issue>) {
    for issue in issues.iter() {
        let message = match issue {
            Issue::AbsolutePath => "Entry has absolute path name.",
            Issue::DataPaddingNotNul => "Data padding byte(s) not nul.",
            Issue::DevMajorWithoutSpecialFile => "Device major is only valid for special file.",
            Issue::DevMinorWithoutSpecialFile => "Device minor is only valid for special file.",
            Issue::DirectoryWithoutSlash => "Directory without trailing slash encountered.",
            Issue::DotDotPath => "Entry has .. as directory part in path name.",
            Issue::EmptyName => "Name field is empty.",
            Issue::EmptyPath => "Name and prefix are empty.",
            Issue::FormatChanged => "Header format changed within archive.",
            Issue::InvalidChecksum => "Checksum does not match.",
            Issue::InvalidMagic => "No tar/ustar magic.",
            Issue::InvalidMode => "Mode is invalid.",
            Issue::InvalidNumber => "Number is invalid.",
            Issue::InvalidTypeFlag => "Typeflag is invalid or not fully portable.",
            Issue::InvalidVersion => "No tar/ustar compatible version.",
            Issue::LargeEntry => "Large entry detected.",
            Issue::LinkIsParent => "A parent path component is a link.",
            Issue::LinkToItself => "Hard link links to itself.",
            Issue::LinknameForNonLink => "Link name for a non-link entry detected.",
            Issue::MultiSlashPath => "Entry has consecutive slashes in path name.",
            Issue::NoDataWithSize => "Entry without data blocks has a size.",
            Issue::NoNumber => "Number field contains no number.",
            Issue::PaxPath => "Pax header possibly defines path.",
            Issue::PaxSize => "Pax header possibly defines size.",
            Issue::ReadOnlyDirectoryWithEntries => "A parent path component is read-only for user.",
            Issue::RegularDirectory => "Directory has no explicit directory typeflag.",
            Issue::TrailingByteNotNul => "Byte(s) after end of archive not nul.",
            Issue::UnterminatedNumber => "Number field has no terminating character.",
            Issue::UnterminatedString => "String field has no terminating character.",
            Issue::UnusedByteNotNul => "Unused byte(s) not nul.",
        };
        eprintln!("=> {}", message);
    }
}
