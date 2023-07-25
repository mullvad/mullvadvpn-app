package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class AppVersionInfo(val supported: Boolean, val suggestedUpgrade: String?) : Parcelable
