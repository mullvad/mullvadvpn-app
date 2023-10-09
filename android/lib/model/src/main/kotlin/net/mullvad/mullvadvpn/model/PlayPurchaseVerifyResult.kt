package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class PlayPurchaseVerifyResult : Parcelable {
    @Parcelize data object Ok : PlayPurchaseVerifyResult()

    @Parcelize
    data class Error(val error: PlayPurchaseVerifyError) : PlayPurchaseVerifyResult()
}
