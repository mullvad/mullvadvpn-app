package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class VoucherSubmissionResult : Parcelable {
    @Parcelize
    data class Ok(val submission: VoucherSubmission) : VoucherSubmissionResult()

    @Parcelize
    data class Error(val error: VoucherSubmissionError) : VoucherSubmissionResult()
}
