package net.mullvad.mullvadvpn.model

sealed class VoucherSubmissionResult {
    class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult()
    class InvalidVoucher : VoucherSubmissionResult()
    class VoucherAlreadyUsed : VoucherSubmissionResult()
    class RpcError : VoucherSubmissionResult()
    class OtherError : VoucherSubmissionResult()
}
