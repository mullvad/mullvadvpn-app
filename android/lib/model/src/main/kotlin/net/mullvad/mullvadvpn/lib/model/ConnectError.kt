package net.mullvad.mullvadvpn.lib.model

sealed interface ConnectError {
    data class Unknown(val throwable: Throwable) : ConnectError

    data object NoVpnPermission : ConnectError
}
