package net.mullvad.mullvadvpn.serviceconnection

sealed class ServiceConnectionState {
    data object Bound : ServiceConnectionState()

    data object Unbound : ServiceConnectionState()
}
