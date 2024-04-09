package net.mullvad.mullvadvpn.model

sealed interface SetWireguardMtuError {
    data class Unknown(val throwable: Throwable) : SetWireguardMtuError
}
