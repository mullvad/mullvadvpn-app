package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Suppress("PARCELABLE_PRIMARY_CONSTRUCTOR_IS_EMPTY")
sealed class VoucherSubmissionResult : Parcelable {
    @Parcelize
    class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult(), Parcelable

    @Parcelize
    class InvalidVoucher : VoucherSubmissionResult(), Parcelable

    @Parcelize
    class VoucherAlreadyUsed : VoucherSubmissionResult(), Parcelable

    @Parcelize
    class RpcError : VoucherSubmissionResult(), Parcelable

    @Parcelize
    class OtherError : VoucherSubmissionResult(), Parcelable
}
