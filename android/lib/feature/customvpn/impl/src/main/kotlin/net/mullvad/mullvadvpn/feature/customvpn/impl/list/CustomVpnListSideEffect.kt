package net.mullvad.mullvadvpn.feature.customvpn.impl.list

sealed interface CustomVpnListSideEffect {
    data object ConfigurationSaved : CustomVpnListSideEffect

    data class FailedToSave(val reason: String) : CustomVpnListSideEffect
}
