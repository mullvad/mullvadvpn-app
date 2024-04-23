use crate::types;
use chrono::TimeZone;
use mullvad_types::account::{AccountData, VoucherSubmission};
#[cfg(target_os = "android")]
use mullvad_types::account::{PlayPurchase, PlayPurchasePaymentToken};

use super::FromProtobufTypeError;

impl From<VoucherSubmission> for types::VoucherSubmission {
    fn from(submission: VoucherSubmission) -> Self {
        types::VoucherSubmission {
            seconds_added: submission.time_added,
            new_expiry: Some(types::Timestamp {
                seconds: submission.new_expiry.timestamp(),
                nanos: 0,
            }),
        }
    }
}

impl TryFrom<types::VoucherSubmission> for VoucherSubmission {
    type Error = FromProtobufTypeError;

    fn try_from(submission: types::VoucherSubmission) -> Result<Self, FromProtobufTypeError> {
        let new_expiry = submission
            .new_expiry
            .ok_or(FromProtobufTypeError::InvalidArgument("missing expiry"))?;
        let ndt =
            chrono::NaiveDateTime::from_timestamp_opt(new_expiry.seconds, new_expiry.nanos as u32)
                .unwrap();

        Ok(VoucherSubmission {
            new_expiry: chrono::Utc.from_utc_datetime(&ndt),
            time_added: submission.seconds_added,
        })
    }
}

impl From<AccountData> for types::AccountData {
    fn from(data: AccountData) -> Self {
        types::AccountData {
            id: data.id,
            expiry: Some(types::Timestamp {
                seconds: data.expiry.timestamp(),
                nanos: 0,
            }),
        }
    }
}

impl TryFrom<types::AccountData> for AccountData {
    type Error = FromProtobufTypeError;

    fn try_from(data: types::AccountData) -> Result<Self, FromProtobufTypeError> {
        let expiry = data
            .expiry
            .ok_or(FromProtobufTypeError::InvalidArgument("missing expiry"))?;
        let ndt =
            chrono::NaiveDateTime::from_timestamp_opt(expiry.seconds, expiry.nanos as u32).unwrap();

        Ok(AccountData {
            id: data.id,
            expiry: chrono::Utc.from_utc_datetime(&ndt),
        })
    }
}

#[cfg(target_os = "android")]
impl TryFrom<types::PlayPurchase> for PlayPurchase {
    type Error = FromProtobufTypeError;

    fn try_from(value: types::PlayPurchase) -> Result<Self, Self::Error> {
        let product_id = value.product_id;
        let purchase_token = value
            .purchase_token
            .ok_or(FromProtobufTypeError::InvalidArgument(
                "Missing purchase token",
            ))?
            .token;

        Ok(Self {
            product_id,
            purchase_token,
        })
    }
}

#[cfg(target_os = "android")]
impl From<PlayPurchasePaymentToken> for types::PlayPurchasePaymentToken {
    fn from(token: PlayPurchasePaymentToken) -> Self {
        Self { token }
    }
}
