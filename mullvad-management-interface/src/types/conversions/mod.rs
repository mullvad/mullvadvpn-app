use std::str::FromStr;

mod custom_tunnel;
mod device;
mod location;
mod net;
pub mod relay_constraints;
mod relay_list;
mod settings;
mod states;
mod version;
mod wireguard;

#[derive(Debug)]
pub enum FromProtobufTypeError {
    InvalidArgument(&'static str),
}

fn bytes_to_pubkey(
    bytes: &[u8],
) -> Result<talpid_types::net::wireguard::PublicKey, FromProtobufTypeError> {
    if bytes.len() != 32 {
        return Err(FromProtobufTypeError::InvalidArgument("invalid public key"));
    }
    let mut public_key = [0; 32];
    public_key.copy_from_slice(&bytes[..32]);
    Ok(talpid_types::net::wireguard::PublicKey::from(public_key))
}

/// Returns `Option<String>`, where an empty string represents `None`.
fn option_from_proto_string(s: String) -> Option<String> {
    match s {
        s if s.is_empty() => None,
        s => Some(s),
    }
}

fn arg_from_str<T: FromStr<Err = E>, E>(
    s: &str,
    invalid_arg_msg: &'static str,
) -> Result<T, FromProtobufTypeError> {
    T::from_str(s).map_err(|_err| FromProtobufTypeError::InvalidArgument(invalid_arg_msg))
}

impl From<FromProtobufTypeError> for crate::Status {
    fn from(err: FromProtobufTypeError) -> Self {
        match err {
            FromProtobufTypeError::InvalidArgument(err) => crate::Status::invalid_argument(err),
        }
    }
}

/// Converts any message to `google.protobuf.Any`.
fn to_proto_any<T: prost::Message>(type_name: &str, message: T) -> prost_types::Any {
    prost_types::Any {
        type_url: format!("type.googleapis.com/{type_name}"),
        value: message.encode_to_vec(),
    }
}

/// Tries to convert a message from `google.protobuf.Any` to `T`.
fn try_from_proto_any<T: prost::Message + Default>(
    type_name: &str,
    any_value: prost_types::Any,
) -> Option<T> {
    if any_value.type_url != format!("type.googleapis.com/{type_name}") {
        return None;
    }
    T::decode(any_value.value.as_slice()).ok()
}
