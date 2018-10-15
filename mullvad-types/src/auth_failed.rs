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
    InvalidReason,
}

impl AuthFailedInner {
    fn from_str(input: &str) -> AuthFailedInner {
        use self::AuthFailedInner::*;
        match parse_string(input) {
            Some(("INVALID_ACCOUNT", _)) => InvalidAccount,
            Some(("EXPIRED_ACCOUNT", _)) => ExpiredAccount,
            Some(("TOO_MANY_CONNECTIONS", _)) => TooManyConnectons,
            Some((unknown_reason, message)) => {
                warn!(
                    "Received AUTH_FAILED message with unknonw reason: {}",
                    input
                );
                Unknown(unknown_reason.to_string(), message.to_string())
            }
            None => {
                warn!("Received invalid AUTH_FAILED message: {}", input);
                InvalidReason
            }
        }
    }


    fn is_invalid(&self) -> bool {
        use self::AuthFailedInner::*;
        match self {
            InvalidReason => true,
            _ => false,
        }
    }
}

impl AuthFailed {
    pub fn from_str(reason: &str) -> AuthFailed {
        AuthFailed {
            reason: AuthFailedInner::from_str(reason),
        }
    }

    pub fn invalid() -> AuthFailed {
        AuthFailed {
            reason: AuthFailedInner::InvalidReason,
        }
    }
}

impl ::std::fmt::Display for AuthFailed {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        use self::AuthFailedInner::*;
        match self.reason {
            InvalidAccount => write!(f, "Account is invalid"),
            ExpiredAccount => write!(f, "Account has expired"),
            TooManyConnectons => write!(f, "Account has too many active connections"),
            Unknown(_, ref reason) => write!(f, "{}", reason),
            InvalidReason => write!(f, "Account authentication failed"),
        }
    }
}

// Expects to take a string like "[INVALID_ACCOUNT] This is not a valid Mullvad account".
// The example input string would be split into:
// * "INVALID_ACCOUNT" - the ID of the failure reason.
// * "This is not a valid Mullvad account" - the human readable message of the failure reason.
// In the case that the message has preceeding whitespace, it will be trimmed.
fn parse_string<'a>(reason: &'a str) -> Option<(&'a str, &'a str)> {
    if !reason.starts_with("[") {
        return None;
    }

    let reason_end_idx = reason.find(']')?;
    let reason_id = &reason[1..reason_end_idx];

    let end = reason.len();
    if reason_end_idx + 1 >= end {
        Some((reason_id, ""))
    } else {
        Some((reason_id, &reason[reason_end_idx + 1..end].trim_left()))
    }
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
            (Some(("REASON REASON", "")), "[REASON REASON]"),
            (Some(("REASON REASON", "A")), "[REASON REASON]A"),
            (None, "incomplete]"),
            (None, ""),
        ];

        for (expected_output, input) in tests.iter() {
            assert_eq!(*expected_output, parse_string(input));
        }
    }

}
