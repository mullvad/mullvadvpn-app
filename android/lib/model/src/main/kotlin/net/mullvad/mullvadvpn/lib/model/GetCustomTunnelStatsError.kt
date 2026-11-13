package net.mullvad.mullvadvpn.lib.model

sealed interface GetCustomTunnelStatsError {
    data class Unknown(val throwable: Throwable) : GetCustomTunnelStatsError
}
