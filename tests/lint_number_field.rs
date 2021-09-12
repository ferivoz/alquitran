use alquitran::issues::Issue;
use alquitran::lint::lint_number_field;

#[test]
fn conforming_number_field() {
    let bytes = b"0012\0\0";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!(10, result.value.unwrap());
}

#[test]
fn empty_number_field() {
    let bytes = b"\0";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::NoNumber, 0)));
    assert!(result.value.is_none());
}

#[test]
fn invalid_number_field() {
    let bytes = b"90\0";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::InvalidNumber, 0)));
    assert!(result.value.is_none());
}

#[test]
fn no_number_field() {
    let bytes = b"";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::NoNumber, 0)));
    assert!(result.value.is_none());
}

#[test]
fn larger_number_in_number_field() {
    let bytes = b"7777777777777777\0";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!(281474976710655, result.value.unwrap());
}

#[test]
fn unterminated_number_field() {
    let bytes = b"0";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnterminatedNumber, 0)));
    assert!(result.value.is_none());
}

#[test]
fn unused_byte_not_nul_in_number_field() {
    let bytes = b"1\0x";
    let result = lint_number_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnusedByteNotNul, 2)));
    assert_eq!(1, result.value.unwrap());
}
