package net.mullvad.mullvadvpn.lib.model

sealed interface SetDaitaSettingsError {
    data class Unknown(val throwable: Throwable) : SetDaitaSettingsError
}
