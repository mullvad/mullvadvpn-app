package net.mullvad.mullvadvpn.model

interface SetAutoConnectError {
    data class Unknown(val throwable: Throwable) : SetAutoConnectError
}
