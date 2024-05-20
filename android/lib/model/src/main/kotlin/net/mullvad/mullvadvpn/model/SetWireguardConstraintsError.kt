package net.mullvad.mullvadvpn.model

sealed interface SetWireguardConstraintsError {
    data class Unknown(val throwable: Throwable) : SetWireguardConstraintsError
}
