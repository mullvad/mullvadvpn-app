use crate::net::Endpoint;
use safelog::Sensitive;
use serde::{Deserialize, Serialize};
use std::{fmt, net::SocketAddr, str::FromStr};

use super::TransportProtocol;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Validation of SOCKS5 username or password failed.
    #[error("Invalid SOCKS5 authentication credentials: {0}")]
    InvalidSocksAuthValues(&'static str),
}

/// Types of bridges that can be used to proxy a connection to a tunnel
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    Shadowsocks,
    Custom,
}

impl fmt::Display for ProxyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        let bridge = match self {
            ProxyType::Shadowsocks => "Shadowsocks",
            ProxyType::Custom => "custom bridge",
        };
        write!(f, "{bridge}")
    }
}

/// Bridge endpoint, broadcast as part of a [`crate::net::TunnelEndpoint`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProxyEndpoint {
    #[serde(flatten)]
    pub endpoint: Endpoint,
    pub proxy_type: ProxyType,
}

/// User customized proxy used for obfuscation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CustomProxy {
    Shadowsocks(Shadowsocks),
    Socks5Local(Socks5Local),
    Socks5Remote(Socks5Remote),
}

impl CustomProxy {
    pub fn get_remote_endpoint(&self) -> ProxyEndpoint {
        match self {
            CustomProxy::Socks5Local(settings) => ProxyEndpoint {
                endpoint: settings.remote_endpoint,
                proxy_type: ProxyType::Custom,
            },
            CustomProxy::Socks5Remote(settings) => ProxyEndpoint {
                endpoint: Endpoint::from_socket_address(settings.endpoint, TransportProtocol::Tcp),
                proxy_type: ProxyType::Custom,
            },
            CustomProxy::Shadowsocks(settings) => ProxyEndpoint {
                endpoint: Endpoint::from_socket_address(settings.endpoint, TransportProtocol::Tcp),
                proxy_type: ProxyType::Shadowsocks,
            },
        }
    }
}

impl From<Socks5Remote> for CustomProxy {
    fn from(value: Socks5Remote) -> Self {
        CustomProxy::Socks5Remote(value)
    }
}

impl From<Socks5Local> for CustomProxy {
    fn from(value: Socks5Local) -> Self {
        CustomProxy::Socks5Local(value)
    }
}

impl From<Shadowsocks> for CustomProxy {
    fn from(value: Shadowsocks) -> Self {
        CustomProxy::Shadowsocks(value)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Shadowsocks {
    pub endpoint: SocketAddr,
    password: Sensitive<String>,
    pub cipher: ShadowsocksCipher,
}

/// Like [shadowsocks_crypto::CipherKind], but implements [Serialize] + [Deserialize].
///
/// A [ShadowsocksCipher] constructed via [ShadowsocksCipher::new] is guaranteed to:
/// - Hnfallibly convert into a [shadowsocks_crypto::CipherKind].
/// - Have the same string representation as [shadowsocks_crypto::CipherKind].
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
pub struct ShadowsocksCipher(String);

impl ShadowsocksCipher {
    /// Validates cipher against all known, supported Shadowsocks ciphers.
    ///
    /// For a list of supported ciphers, see [Self::list].
    pub fn new(cipher: &str) -> Result<Self, shadowsocks_crypto::kind::ParseCipherKindError> {
        let cipher = shadowsocks_crypto::CipherKind::from_str(cipher)?;
        Ok(Self(cipher.to_string()))
    }

    pub fn kind(self) -> shadowsocks_crypto::CipherKind {
        shadowsocks_crypto::CipherKind::from_str(&self.0).unwrap()
    }

    pub fn list() -> &'static [&'static str] {
        shadowsocks_crypto::available_ciphers()
    }

    /// Return all unique Shadowsocks ciphers.
    ///
    /// Some ciphers from [`ShadowsocksCipher::list`] share the same internal representation, which
    /// means that parsing the list of strings may yield duplicate elements.
    pub fn enumerate() -> Vec<Self> {
        use itertools::Itertools;
        ShadowsocksCipher::list()
            .iter()
            .map(|cipher| ShadowsocksCipher::new(cipher).unwrap())
            .unique()
            .collect()
    }
}

impl<'de> Deserialize<'de> for ShadowsocksCipher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CipherVisitor(std::marker::PhantomData<Self>);
        impl serde::de::Visitor<'_> for CipherVisitor {
            type Value = ShadowsocksCipher;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(formatter, "a cipher string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                value
                    .parse::<Self::Value>()
                    .map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(CipherVisitor(std::marker::PhantomData))
    }
}

impl core::fmt::Display for ShadowsocksCipher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<shadowsocks_crypto::CipherKind> for ShadowsocksCipher {
    fn from(cipher: shadowsocks_crypto::CipherKind) -> Self {
        ShadowsocksCipher::new(&cipher.to_string()).unwrap()
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse Shadowsocks cipher: {0}")]
pub struct ParseCipherKindError(shadowsocks_crypto::kind::ParseCipherKindError);

impl FromStr for ShadowsocksCipher {
    type Err = ParseCipherKindError;

    fn from_str(cipher: &str) -> Result<Self, Self::Err> {
        Self::new(cipher).map_err(ParseCipherKindError)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Local {
    pub remote_endpoint: Endpoint,
    /// Port on localhost where the SOCKS5-proxy listens to.
    pub local_port: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Socks5Remote {
    pub endpoint: SocketAddr,
    pub auth: Option<SocksAuth>,
}

/// A valid SOCKS5 username/password authentication according to
/// RFC 1929: <https://datatracker.ietf.org/doc/html/rfc1929>.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SocksAuth {
    username: Sensitive<String>,
    password: Sensitive<String>,
}

impl SocksAuth {
    /// Validate a SOCKS5 username/password based authentication.
    ///
    /// # Examples
    ///
    /// A valid username and password both have to be between 1 and 255 bytes.
    ///
    /// ```
    /// use talpid_types::net::proxy::SocksAuth;
    ///
    /// let valid_auth = SocksAuth::new("FooBar".to_string(), "hunter2".to_string());
    /// assert!(valid_auth.is_ok());
    /// ```
    ///
    /// An empty username or password is not valid.
    ///
    /// ```
    /// use talpid_types::net::proxy::SocksAuth;
    ///
    /// // An empty username is not a valid username.
    /// let invalid_username = SocksAuth::new("".to_string(), "hunter2".to_string());
    /// assert!(invalid_username.is_err());
    ///
    /// // Likeweise, an empty password is not a valid password.
    /// let invalid_password = SocksAuth::new("FooBar".to_string(), "".to_string());
    /// assert!(invalid_password.is_err());
    /// ```
    ///
    /// The upper limit for a valid username and password is 255 bytes
    ///
    /// ```
    /// use talpid_types::net::proxy::SocksAuth;
    ///
    /// let max_valid_username = SocksAuth::new("x".repeat(255), "hunter2".to_string());
    /// assert!(max_valid_username.is_ok());
    ///
    /// let too_long_username = SocksAuth::new("x".repeat(256), "hunter2".to_string());
    /// assert!(too_long_username.is_err());
    ///
    /// let max_valid_password = SocksAuth::new("FooBar".to_string(), "x".repeat(255));
    /// assert!(max_valid_username.is_ok());
    ///
    /// let too_long_password = SocksAuth::new("FooBar".to_string(), "x".repeat(256));
    /// assert!(too_long_username.is_err());
    /// ```
    pub fn new(username: String, password: String) -> Result<Self, Error> {
        if !(1..=255).contains(&password.len()) {
            return Err(Error::InvalidSocksAuthValues(
                "Password length should between 1 and 255 bytes",
            ));
        }
        if !(1..=255).contains(&username.len()) {
            return Err(Error::InvalidSocksAuthValues(
                "Username length should between 1 and 255 bytes",
            ));
        }

        Ok(SocksAuth {
            username: username.into(),
            password: password.into(),
        })
    }

    /// Read the username.
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Read the password.
    pub fn password(&self) -> &str {
        &self.password
    }
}

impl Shadowsocks {
    pub fn new<I: Into<SocketAddr>>(
        endpoint: I,
        cipher: ShadowsocksCipher,
        password: String,
    ) -> Self {
        Shadowsocks {
            endpoint: endpoint.into(),
            password: password.into(),
            cipher,
        }
    }

    /// Get a reference to the password in plaintext.
    ///
    /// Caution: DO NOT LOG THIS ANYWHERE.
    pub fn plaintext_password(&self) -> &str {
        self.password.as_inner()
    }
}

impl Socks5Local {
    pub fn new<I: Into<SocketAddr>>(remote_endpoint: I, local_port: u16) -> Self {
        let transport_protocol = TransportProtocol::Tcp;
        Self::new_with_transport_protocol(remote_endpoint, local_port, transport_protocol)
    }

    pub fn new_with_transport_protocol<I: Into<SocketAddr>>(
        remote_endpoint: I,
        local_port: u16,
        transport_protocol: TransportProtocol,
    ) -> Self {
        let remote_endpoint =
            Endpoint::from_socket_address(remote_endpoint.into(), transport_protocol);
        Self {
            remote_endpoint,
            local_port,
        }
    }
}

impl Socks5Remote {
    pub fn new<I: Into<SocketAddr>>(endpoint: I) -> Self {
        Self {
            endpoint: endpoint.into(),
            auth: None,
        }
    }

    pub fn new_with_authentication<I: Into<SocketAddr>>(
        endpoint: I,
        authentication: SocksAuth,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            auth: Some(authentication),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Iterate all known ciphers and ensure [ShadowsocksCipher] can be constructed for all of them.
    #[test]
    fn parse_shadowsocks_ciphers() {
        for cipher in ShadowsocksCipher::list() {
            let cipher =
                ShadowsocksCipher::new(cipher).expect("{cipher} must be a valid ShadowsocksCipher");
            // It *must be* infallible to convert a ShadowsocksCipher back to the CipherKind type.
            let _kind = cipher.kind();
        }
    }

    #[test]
    /// Snapshot all Shadowsocks ciphers. They might change between versions of shadowsocks-rs, and
    /// we consider a removal of any cipher a breaking change.
    fn shadowsocks_ciphers() {
        insta::assert_debug_snapshot!(ShadowsocksCipher::list());
        insta::assert_debug_snapshot!(ShadowsocksCipher::enumerate());
    }
}
