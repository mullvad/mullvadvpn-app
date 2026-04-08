package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import java.net.InetAddress
import kotlinx.parcelize.Parcelize

@Parcelize data class Quic(val inAddresses: List<InetAddress>) : Parcelable
