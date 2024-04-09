package net.mullvad.mullvadvpn.model

sealed interface SetWireguardQuantumResistantError {
    data class Unknown(val throwable: Throwable) : SetWireguardQuantumResistantError
}
