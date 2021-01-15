package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class VoucherSubmissionResult : Parcelable {
    @Parcelize
    class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult(), Parcelable

    @Parcelize
    object InvalidVoucher : VoucherSubmissionResult(), Parcelable

    @Parcelize
    object VoucherAlreadyUsed : VoucherSubmissionResult(), Parcelable

    @Parcelize
    object RpcError : VoucherSubmissionResult(), Parcelable

    @Parcelize
    object OtherError : VoucherSubmissionResult(), Parcelable
}
