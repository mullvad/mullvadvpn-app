use std::str::FromStr;

mod access_method;
mod account;
mod custom_list;
mod custom_tunnel;
mod device;
mod features;
mod location;
mod logging;
mod net;
#[cfg(feature = "personal-vpn")]
mod personal_vpn;
pub mod relay_constraints;
mod relay_list;
mod relay_selector;
mod settings;
#[cfg(target_os = "windows")]
mod split_tunnel;
mod states;
mod version;
mod wireguard;

#[derive(thiserror::Error, Debug)]
pub enum FromProtobufTypeError {
    #[error("Invalid argument for type conversion: {0}")]
    InvalidArgument(String),
}

impl FromProtobufTypeError {
    /// Shorthand for `FromProtobufTypeError::invalid_argument(String::from(<msg>))`.
    #[inline(always)]
    pub fn invalid_argument(msg: impl Into<String>) -> FromProtobufTypeError {
        FromProtobufTypeError::InvalidArgument(msg.into())
    }
}

fn bytes_to_pubkey(
    bytes: &[u8],
) -> Result<talpid_types::net::wireguard::PublicKey, FromProtobufTypeError> {
    Ok(talpid_types::net::wireguard::PublicKey::from(
        *bytes_to_wg_key(bytes, "invalid public key")?,
    ))
}

fn bytes_to_privkey(
    bytes: &[u8],
) -> Result<talpid_types::net::wireguard::PrivateKey, FromProtobufTypeError> {
    Ok(talpid_types::net::wireguard::PrivateKey::from(
        *bytes_to_wg_key(bytes, "invalid private key")?,
    ))
}

fn bytes_to_wg_key<'a>(
    bytes: &'a [u8],
    error_msg: &'static str,
) -> Result<&'a [u8; 32], FromProtobufTypeError> {
    <&[u8; 32]>::try_from(bytes).map_err(|_| FromProtobufTypeError::invalid_argument(error_msg))
}

fn arg_from_str<T: FromStr<Err = E>, E>(
    s: &str,
    invalid_arg_msg: &'static str,
) -> Result<T, FromProtobufTypeError> {
    T::from_str(s).map_err(|_err| FromProtobufTypeError::invalid_argument(invalid_arg_msg))
}

impl From<FromProtobufTypeError> for crate::Status {
    fn from(err: FromProtobufTypeError) -> Self {
        match err {
            FromProtobufTypeError::InvalidArgument(err) => crate::Status::invalid_argument(err),
        }
    }
}
