package net.mullvad.mullvadvpn.lib.model

sealed interface NotificationTunnelState {
    data class Disconnected(val hasVpnPermission: Boolean) : NotificationTunnelState

    data object Connecting : NotificationTunnelState

    data object Connected : NotificationTunnelState

    data object Reconnecting : NotificationTunnelState

    data object Disconnecting : NotificationTunnelState

    sealed interface Error : NotificationTunnelState {
        data object DeviceOffline : Error

        data object Blocking : Error

        data object VpnPermissionDenied : Error

        data object AlwaysOnVpn : Error

        data object Critical : Error
    }
}
