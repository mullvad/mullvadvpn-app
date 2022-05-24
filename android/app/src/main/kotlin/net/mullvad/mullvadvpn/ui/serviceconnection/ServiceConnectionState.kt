package net.mullvad.mullvadvpn.ui.serviceconnection

sealed class ServiceConnectionState {
    data class ConnectedReady(val container: ServiceConnectionContainer) : ServiceConnectionState()

    data class ConnectedNotReady(val container: ServiceConnectionContainer) :
        ServiceConnectionState()

    object Disconnected : ServiceConnectionState()

    fun readyContainer(): ServiceConnectionContainer? {
        return (this as? ConnectedReady)?.container
    }
}
