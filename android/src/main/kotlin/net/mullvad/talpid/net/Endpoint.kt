package net.mullvad.talpid.net

import android.os.Parcelable
import java.net.InetSocketAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class Endpoint(val address: InetSocketAddress, val protocol: TransportProtocol) : Parcelable
