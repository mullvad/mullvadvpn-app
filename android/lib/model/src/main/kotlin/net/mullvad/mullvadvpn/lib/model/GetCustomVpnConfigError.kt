package net.mullvad.mullvadvpn.lib.model

sealed interface GetCustomVpnConfigError {
    object Unknown : GetCustomVpnConfigError
}
