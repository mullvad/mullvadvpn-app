package net.mullvad.mullvadvpn.lib.model

sealed interface UpdateRelayLocationsError {
    data class Unknown(val throwable: Throwable) : UpdateRelayLocationsError
}
