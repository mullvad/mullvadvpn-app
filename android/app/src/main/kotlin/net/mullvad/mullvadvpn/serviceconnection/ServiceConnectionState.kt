package net.mullvad.mullvadvpn.serviceconnection

sealed interface ServiceConnectionState {
    data object Bound : ServiceConnectionState

    data object Unbound : ServiceConnectionState
}
