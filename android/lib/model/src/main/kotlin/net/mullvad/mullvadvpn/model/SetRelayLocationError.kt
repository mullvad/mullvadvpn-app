package net.mullvad.mullvadvpn.model

interface SetRelayLocationError {
    data class Unknown(val throwable: Throwable) : SetRelayLocationError
}
