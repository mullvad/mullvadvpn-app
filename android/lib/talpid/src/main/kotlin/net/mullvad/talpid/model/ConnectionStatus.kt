package net.mullvad.talpid.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class ConnectionStatus(val ipv4: Boolean, val ipv6: Boolean) : Parcelable
