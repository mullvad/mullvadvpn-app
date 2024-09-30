package net.mullvad.talpid.model

data class NetworkInfo(val hasIpV4: Boolean, val hasIpV6: Boolean) {
    val isConnected = hasIpV4 || hasIpV6
}
