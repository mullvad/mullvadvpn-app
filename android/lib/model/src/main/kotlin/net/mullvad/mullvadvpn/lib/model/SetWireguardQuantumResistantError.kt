package net.mullvad.mullvadvpn.lib.model

sealed interface SetWireguardQuantumResistantError {
    data class Unknown(val throwable: Throwable) : SetWireguardQuantumResistantError
}
