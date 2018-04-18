use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::iter::repeat;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

/// Common UTF-16 code points.
mod utf16 {
    pub const DOUBLEQUOTE: u16 = '"' as u16;
    pub const BACKSLASH: u16 = '\\' as u16;
    pub const SPACE: u16 = ' ' as u16;
    pub const LINEFEED: u16 = '\n' as u16;
    pub const HTAB: u16 = '\t' as u16;
    pub const VTAB: u16 = 0x000B; // '\v'
}

/// Loselessly escape shell arguments on Windows.
///
/// Inspired by https://blogs.msdn.microsoft.com/twistylittlepassagesallalike/2011/04/23/everyone-quotes-command-line-arguments-the-wrong-way/.
/// Heavily based on https://github.com/sfackler/shell-escape
pub fn escape(s: Cow<OsStr>) -> Cow<OsStr> {
    static ESCAPE_CHARS: &'static [u16] = &[
        utf16::DOUBLEQUOTE,
        utf16::SPACE,
        utf16::LINEFEED,
        utf16::HTAB,
        utf16::VTAB,
    ];
    let needs_escape = s.is_empty() || s.encode_wide().any(|ref c| ESCAPE_CHARS.contains(c));
    if !needs_escape {
        return s;
    }

    let mut escaped_wide_string: Vec<u16> = Vec::with_capacity(s.len() + 2);
    escaped_wide_string.push(utf16::DOUBLEQUOTE);

    let mut chars = s.encode_wide().peekable();
    loop {
        let mut num_slashes = 0;
        while let Some(&utf16::BACKSLASH) = chars.peek() {
            chars.next();
            num_slashes += 1;
        }

        match chars.next() {
            Some(utf16::DOUBLEQUOTE) => {
                escaped_wide_string.extend(repeat(utf16::BACKSLASH).take(num_slashes * 2 + 1));
                escaped_wide_string.push(utf16::DOUBLEQUOTE);
            }
            Some(c) => {
                escaped_wide_string.extend(repeat(utf16::BACKSLASH).take(num_slashes));
                escaped_wide_string.push(c);
            }
            None => {
                escaped_wide_string.extend(repeat(utf16::BACKSLASH).take(num_slashes * 2));
                break;
            }
        }
    }

    escaped_wide_string.push(utf16::DOUBLEQUOTE);

    Cow::Owned(OsString::from_wide(&escaped_wide_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_escape() {
        assert_eq!(
            escape(Cow::Borrowed(OsStr::new("--aaa=bbb-ccc"))),
            OsStr::new("--aaa=bbb-ccc")
        );
    }

    #[test]
    fn test_escape_empty_argument() {
        assert_eq!(escape(Cow::Borrowed(OsStr::new(""))), OsStr::new(r#""""#));
    }

    #[test]
    fn test_escape_argument_with_spaces() {
        assert_eq!(
            escape(Cow::Borrowed(OsStr::new("linker=gcc -L/foo -Wl,bar"))),
            OsStr::new(r#""linker=gcc -L/foo -Wl,bar""#)
        );
    }

    #[test]
    fn test_escape_nested_quotes() {
        assert_eq!(
            escape(Cow::Borrowed(OsStr::new(r#"--features="default""#))),
            OsStr::new(r#""--features=\"default\"""#)
        );
    }


    #[test]
    fn test_escape_multiple_backslashes_and_nested_quotes() {
        assert_eq!(
            escape(Cow::Borrowed(OsStr::new(r#"hello \\\"quote\\\""#))),
            OsStr::new(r#""hello \\\\\\\"quote\\\\\\\"""#)
        );
    }

    // Input:
    // child.exe "\some\directory with\spaces\" argument2
    //
    // Parsed as:
    // 0: [child.exe]
    // 1: [\some\directory with\spaces" argument2]
    #[test]
    fn test_escape_trailing_backslash() {
        assert_eq!(
            escape(Cow::Borrowed(OsStr::new(r#"\some\directory with\spaces\"#))),
            OsStr::new(r#""\some\directory with\spaces\\""#)
        );
    }
}
