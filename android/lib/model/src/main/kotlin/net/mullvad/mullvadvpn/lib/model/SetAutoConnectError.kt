package net.mullvad.mullvadvpn.lib.model

sealed interface SetAutoConnectError {
    data class Unknown(val throwable: Throwable) : SetAutoConnectError
}
