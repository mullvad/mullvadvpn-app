package net.mullvad.mullvadvpn.lib.model

sealed interface SetWireguardMtuError {
    data class Unknown(val throwable: Throwable) : SetWireguardMtuError
}
