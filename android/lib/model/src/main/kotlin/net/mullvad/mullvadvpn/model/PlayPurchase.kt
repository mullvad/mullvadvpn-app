package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class PlayPurchase(val productId: String, val purchaseToken: String) : Parcelable
