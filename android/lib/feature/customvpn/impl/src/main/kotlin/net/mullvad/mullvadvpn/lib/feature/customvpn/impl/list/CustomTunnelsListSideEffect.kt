package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.list

sealed interface CustomTunnelsListSideEffect {
    data object ConfigurationSaved : CustomTunnelsListSideEffect

    data class FailedToSave(val reason: String) : CustomTunnelsListSideEffect
}
