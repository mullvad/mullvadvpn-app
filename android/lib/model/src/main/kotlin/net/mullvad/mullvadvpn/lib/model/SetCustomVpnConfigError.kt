package net.mullvad.mullvadvpn.lib.model

sealed interface SetCustomVpnConfigError {
    data class Unknown(val throwable: Throwable) : SetCustomVpnConfigError
}
