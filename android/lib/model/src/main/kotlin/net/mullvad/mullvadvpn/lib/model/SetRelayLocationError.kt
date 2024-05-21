package net.mullvad.mullvadvpn.lib.model

sealed interface SetRelayLocationError {
    data class Unknown(val throwable: Throwable) : SetRelayLocationError
}
