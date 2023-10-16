package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class PlayPurchaseInitResult : Parcelable {
    @Parcelize data class Ok(val obfuscatedId: String) : PlayPurchaseInitResult()

    @Parcelize data class Error(val error: PlayPurchaseInitError) : PlayPurchaseInitResult()
}
