use regex::Regex;

#[derive(Debug)]
pub struct AuthFailed {
    reason: AuthFailedInner,
}

#[derive(Debug)]
enum AuthFailedInner {
    InvalidAccount,
    ExpiredAccount,
    TooManyConnectons,
    Unknown(String, String),
}

// These strings should match up with gui/packages/desktop/src/renderer/lib/auth-failure.js
const INVALID_ACCOUNT_MSG: &str = "You've logged in with an account number that is not valid. Please log out and try another one.";
const EXPIRED_ACCOUNT_MSG: &str = "You have no more VPN time left on this account. Please log in on our website to buy more credit.";
const TOO_MANY_CONNECTIONS_MSG: &str = "This account has too many simultaneous connections. Disconnect another device or try connecting again shortly.";

impl AuthFailedInner {
    fn from_str(input: &str) -> AuthFailedInner {
        use self::AuthFailedInner::*;
        match parse_string(input) {
            Some(("INVALID_ACCOUNT", _)) => InvalidAccount,
            Some(("EXPIRED_ACCOUNT", _)) => ExpiredAccount,
            Some(("TOO_MANY_CONNECTIONS", _)) => TooManyConnectons,
            Some((unknown_reason, message)) => {
                warn!(
                    "Received AUTH_FAILED message with unknown reason: {}",
                    input
                );
                Unknown(unknown_reason.to_string(), message.to_string())
            }
            None => {
                warn!("Received invalid AUTH_FAILED message: {}", input);
                Unknown("UNKNOWN".to_string(), input.to_string())
            }
        }
    }
}

impl AuthFailed {
    pub fn from_str(reason: &str) -> AuthFailed {
        AuthFailed {
            reason: AuthFailedInner::from_str(reason),
        }
    }
}

impl ::std::fmt::Display for AuthFailed {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::AuthFailedInner::*;
        match self.reason {
            InvalidAccount => write!(f, "{}", INVALID_ACCOUNT_MSG),
            ExpiredAccount => write!(f, "{}", EXPIRED_ACCOUNT_MSG),
            TooManyConnectons => write!(f, "{}", TOO_MANY_CONNECTIONS_MSG),
            Unknown(_, ref reason) => write!(f, "{}", reason),
        }
    }
}

// Expects to take a string like "[INVALID_ACCOUNT] This is not a valid Mullvad account".
// The example input string would be split into:
// * "INVALID_ACCOUNT" - the ID of the failure reason.
// * "This is not a valid Mullvad account" - the human readable message of the failure reason.
// In the case that the message has preceeding whitespace, it will be trimmed.
fn parse_string<'a>(reason: &'a str) -> Option<(&'a str, &'a str)> {
    lazy_static! {
        static ref REASON_REGEX: Regex = Regex::new(r"^\[(\w+)\]\s*(.*)$").unwrap();
    }
    let captures = REASON_REGEX.captures(reason)?;
    let reason = captures.get(1).map(|m| m.as_str())?;
    let message = captures.get(2).map(|m| m.as_str())?;
    Some((reason, message))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parsing() {
        let tests = vec![
            (Some(("INVALID_ACCOUNT", "This is not a valid Mullvad account" )),
                "[INVALID_ACCOUNT] This is not a valid Mullvad account"),
            (Some(("EXPIRED_ACCOUNT", "This account has no time left")),
             "[EXPIRED_ACCOUNT] This account has no time left"),
            (Some(("TOO_MANY_CONNECTIONS", "This Mullvad account is already used by the maximum number of simultaneous connections")),
            "[TOO_MANY_CONNECTIONS] This Mullvad account is already used by the maximum number of simultaneous connections"),
            (None, "[Incomplete String"),
            (Some(("REASON_REASON", "")), "[REASON_REASON]"),
            (Some(("REASON_REASON", "A")), "[REASON_REASON]A"),
            (None, "incomplete]"),
            (None, ""),
        ];

        for (expected_output, input) in tests.iter() {
            assert_eq!(*expected_output, parse_string(input));
        }
    }

}
