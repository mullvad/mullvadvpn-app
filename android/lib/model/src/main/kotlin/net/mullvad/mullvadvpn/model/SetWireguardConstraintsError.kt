package net.mullvad.mullvadvpn.model

interface SetWireguardConstraintsError {
    data class Unknown(val throwable: Throwable) : SetWireguardConstraintsError
}
