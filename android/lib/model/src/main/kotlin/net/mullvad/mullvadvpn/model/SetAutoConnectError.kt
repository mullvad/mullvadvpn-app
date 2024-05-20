package net.mullvad.mullvadvpn.model

sealed interface SetAutoConnectError {
    data class Unknown(val throwable: Throwable) : SetAutoConnectError
}
