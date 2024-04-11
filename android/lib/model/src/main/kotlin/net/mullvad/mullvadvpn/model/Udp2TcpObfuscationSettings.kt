package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

data class Udp2TcpObfuscationSettings(val port: Constraint<Port>)
