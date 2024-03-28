package net.mullvad.mullvadvpn.ui.serviceconnection

sealed class ServiceConnectionState {
    data object Bound : ServiceConnectionState()

    data object Binding : ServiceConnectionState()

    data object Unbound : ServiceConnectionState()
}
