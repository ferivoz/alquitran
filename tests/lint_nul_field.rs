use alquitran::issues::Issue;
use alquitran::lint::lint_nul_field;

#[test]
fn conforming_nul_field() {
    let bytes = b"\0\0";
    let result = lint_nul_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert!(result.value.unwrap());
}

#[test]
fn no_nul_field() {
    let bytes = b"";
    let result = lint_nul_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert!(result.value.unwrap());
}

#[test]
fn unused_byte_not_nul_in_nul_field() {
    let bytes = b"1\0";
    let result = lint_nul_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnusedByteNotNul, 0)));
    assert!(!result.value.unwrap());
}
