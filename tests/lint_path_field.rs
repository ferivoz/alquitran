use alquitran::issues::Hint;
use alquitran::issues::Issue;
use alquitran::lint::lint_path_field;

#[test]
fn conforming_path_field() {
    let bytes = b"Portable/Name.123\0\0";
    let result = lint_path_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("Portable/Name.123".as_bytes(), result.value.unwrap());
}

#[test]
fn empty_path_field() {
    let bytes = b"\0";
    let result = lint_path_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("".as_bytes(), result.value.unwrap());
}

#[test]
fn no_path_field() {
    let bytes = b"";
    let result = lint_path_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("".as_bytes(), result.value.unwrap());
}

#[test]
fn unportable_char_in_path_field() {
    let bytes = b"user@host\0";
    let result = lint_path_field(&bytes[..]);
    assert_eq!(1, result.hints.len());
    assert!(result.hints.contains(&(Hint::UnportableCharInPath, 4)));
    assert!(result.issues.is_empty());
    assert_eq!("user@host".as_bytes(), result.value.unwrap());
}

#[test]
fn unterminated_path_field() {
    let bytes = b"path";
    let result = lint_path_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("path".as_bytes(), result.value.unwrap());
}

#[test]
fn unused_byte_not_nul_in_path_field() {
    let bytes = b"path\0x";
    let result = lint_path_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnusedByteNotNul, 5)));
    assert_eq!("path".as_bytes(), result.value.unwrap());
}
