use alquitran::issues::Hint;
use alquitran::issues::Issue;
use alquitran::lint::lint_string_field;

#[test]
fn conforming_string_field() {
    let bytes = b"Portable_String.123\0\0";
    let result = lint_string_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("Portable_String.123".as_bytes(), result.value.unwrap());
}

#[test]
fn empty_string_field() {
    let bytes = b"\0";
    let result = lint_string_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert_eq!("".as_bytes(), result.value.unwrap());
}

#[test]
fn no_string_field() {
    let bytes = b"";
    let result = lint_string_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert!(result.issues.is_empty());
    assert!(result.value.is_none());
}

#[test]
fn unportable_char_in_string_field() {
    let bytes = b"user@host\0";
    let result = lint_string_field(&bytes[..]);
    assert_eq!(1, result.hints.len());
    assert!(result.hints.contains(&(Hint::UnportableCharInString, 4)));
    assert!(result.issues.is_empty());
    assert_eq!("user@host".as_bytes(), result.value.unwrap());
}

#[test]
fn unterminated_string_field() {
    let bytes = b"string";
    let result = lint_string_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnterminatedString, 5)));
    assert!(result.value.is_none());
}

#[test]
fn unused_byte_not_nul_in_string_field() {
    let bytes = b"string\0x";
    let result = lint_string_field(&bytes[..]);
    assert!(result.hints.is_empty());
    assert_eq!(1, result.issues.len());
    assert!(result.issues.contains(&(Issue::UnusedByteNotNul, 7)));
    assert_eq!("string".as_bytes(), result.value.unwrap());
}
