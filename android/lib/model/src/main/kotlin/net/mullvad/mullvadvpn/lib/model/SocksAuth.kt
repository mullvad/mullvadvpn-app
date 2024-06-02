package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class SocksAuth(val username: String, val password: String) : Parcelable
