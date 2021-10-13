package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class VoucherSubmissionError : Parcelable {
    InvalidVoucher,
    VoucherAlreadyUsed,
    RpcError,
    OtherError,
}
