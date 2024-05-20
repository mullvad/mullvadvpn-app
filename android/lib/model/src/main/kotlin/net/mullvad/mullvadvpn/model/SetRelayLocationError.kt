package net.mullvad.mullvadvpn.model

sealed interface SetRelayLocationError {
    data class Unknown(val throwable: Throwable) : SetRelayLocationError
}
