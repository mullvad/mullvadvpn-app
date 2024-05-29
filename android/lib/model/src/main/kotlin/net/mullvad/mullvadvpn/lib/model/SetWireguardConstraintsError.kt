package net.mullvad.mullvadvpn.lib.model

sealed interface SetWireguardConstraintsError {
    data class Unknown(val throwable: Throwable) : SetWireguardConstraintsError
}
