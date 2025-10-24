package net.mullvad.mullvadvpn.lib.model

sealed interface NotificationTunnelState {
    data class Disconnected(val prepareError: PrepareError?) : NotificationTunnelState

    data class Connecting(val location: GeoIpLocation?) : NotificationTunnelState

    data class Connected(val location: GeoIpLocation?) : NotificationTunnelState

    data object Blocking : NotificationTunnelState

    data object Disconnecting : NotificationTunnelState

    sealed interface Error : NotificationTunnelState {
        data object DeviceOffline : Error

        data object Blocked : Error

        data object VpnPermissionDenied : Error

        data class AlwaysOnVpn(val appName: String) : Error

        data object LegacyLockdown : Error

        data object Critical : Error
    }
}
