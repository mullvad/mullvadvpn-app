use lazy_static::lazy_static;
use regex::Regex;
use std::fmt;
use talpid_types::tunnel::ErrorStateCause;

/// Used to parse [`talpid_types::tunnel::ErrorStateCause::AuthFailed`], which may be returned
/// in [`talpid_types::tunnel::ErrorStateCause`] when there is a failure to authenticate
/// with a remote server.
#[derive(Debug)]
pub enum AuthFailed {
    InvalidAccount,
    ExpiredAccount,
    TooManyConnections,
    Unknown,
}

// These strings should match up with gui/packages/desktop/src/renderer/lib/auth-failure.js
const INVALID_ACCOUNT_MSG: &str = "You've logged in with an account number that is not valid. Please log out and try another one.";
const EXPIRED_ACCOUNT_MSG: &str = "You have no more VPN time left on this account. Please log in on our website to buy more credit.";
const TOO_MANY_CONNECTIONS_MSG: &str = "This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.";
const UNKNOWN_MSG: &str = "Unknown error.";

impl<'a> From<&'a str> for AuthFailed {
    fn from(reason: &'a str) -> AuthFailed {
        use AuthFailed::*;
        match parse_string(reason) {
            Some("INVALID_ACCOUNT") => InvalidAccount,
            Some("EXPIRED_ACCOUNT") => ExpiredAccount,
            Some("TOO_MANY_CONNECTIONS") => TooManyConnections,
            Some(fail_id) => {
                log::warn!(
                    "Received AUTH_FAILED message with unknown failure ID: {}",
                    fail_id
                );
                Unknown
            }
            None => {
                log::warn!("Received invalid AUTH_FAILED message: {}", reason);
                Unknown
            }
        }
    }
}

#[derive(Debug)]
pub struct UnexpectedErrorStateCause(());

impl TryFrom<&ErrorStateCause> for AuthFailed {
    type Error = UnexpectedErrorStateCause;

    fn try_from(cause: &ErrorStateCause) -> Result<Self, Self::Error> {
        match cause {
            ErrorStateCause::AuthFailed(Some(reason)) => Ok(AuthFailed::from(reason.as_str())),
            ErrorStateCause::AuthFailed(None) => Ok(AuthFailed::Unknown),
            _ => Err(UnexpectedErrorStateCause(())),
        }
    }
}

impl fmt::Display for AuthFailed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AuthFailed::*;
        match self {
            InvalidAccount => f.write_str(INVALID_ACCOUNT_MSG),
            ExpiredAccount => f.write_str(EXPIRED_ACCOUNT_MSG),
            TooManyConnections => f.write_str(TOO_MANY_CONNECTIONS_MSG),
            Unknown => f.write_str(UNKNOWN_MSG),
        }
    }
}

// Expects to take a string like "[INVALID_ACCOUNT] This is not a valid Mullvad account".
// The example input string would be split into:
// * "INVALID_ACCOUNT" - the ID of the failure reason.
// * "This is not a valid Mullvad account" - human-readable message (ignored).
// In the case that the message has preceeding whitespace, it will be trimmed.
fn parse_string(reason: &str) -> Option<&str> {
    lazy_static! {
        static ref REASON_REGEX: Regex = Regex::new(r"^\[(\w+)\]\s*").unwrap();
    }
    let captures = REASON_REGEX.captures(reason)?;
    captures.get(1).map(|m| m.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let tests = vec![
            (Some("INVALID_ACCOUNT"),
                "[INVALID_ACCOUNT] This is not a valid Mullvad account"),
            (Some("EXPIRED_ACCOUNT"),
             "[EXPIRED_ACCOUNT] This account has no time left"),
            (Some("TOO_MANY_CONNECTIONS"),
            "[TOO_MANY_CONNECTIONS] This Mullvad account is already used by the maximum number of simultaneous connections"),
            (None, "[Incomplete String"),
            (Some("REASON_REASON"), "[REASON_REASON]"),
            (Some("REASON_REASON"), "[REASON_REASON]A"),
            (None, "incomplete]"),
            (None, ""),
        ];

        for (expected_output, input) in tests.iter() {
            assert_eq!(*expected_output, parse_string(input));
        }
    }
}
