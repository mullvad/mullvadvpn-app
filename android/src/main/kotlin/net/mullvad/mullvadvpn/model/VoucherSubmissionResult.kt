package net.mullvad.mullvadvpn.model

sealed class VoucherSubmissionResult {
    class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult()
    object InvalidVoucher : VoucherSubmissionResult()
    object VoucherAlreadyUsed : VoucherSubmissionResult()
    object RpcError : VoucherSubmissionResult()
    object OtherError : VoucherSubmissionResult()
}
