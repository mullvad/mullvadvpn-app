package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class VoucherSubmissionResult : Parcelable {
    @Parcelize
    data class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult()

    @Parcelize
    object InvalidVoucher : VoucherSubmissionResult()

    @Parcelize
    object VoucherAlreadyUsed : VoucherSubmissionResult()

    @Parcelize
    object RpcError : VoucherSubmissionResult()

    @Parcelize
    object OtherError : VoucherSubmissionResult()
}
