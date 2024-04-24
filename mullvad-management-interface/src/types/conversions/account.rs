use crate::types;
use chrono::DateTime;
use mullvad_types::account::{AccountData, VoucherSubmission};

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

        let new_expiry = DateTime::from_timestamp(new_expiry.seconds, new_expiry.nanos as u32)
            .ok_or(FromProtobufTypeError::InvalidArgument("invalid timestamp"))?;

        Ok(VoucherSubmission {
            new_expiry,
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

        let expiry = DateTime::from_timestamp(expiry.seconds, expiry.nanos as u32)
            .ok_or(FromProtobufTypeError::InvalidArgument("invalid timestamp"))?;

        Ok(AccountData {
            id: data.id,
            expiry,
        })
    }
}
