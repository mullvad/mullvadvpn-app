package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import java.net.InetSocketAddress
import kotlinx.parcelize.Parcelize

@Parcelize
data class ApiEndpoint(
    val address: InetSocketAddress,
    val disableAddressCache: Boolean,
    val disableTls: Boolean,
    val forceDirectConnection: Boolean
) : Parcelable
