package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

sealed interface CustomVpnSideEffect {

    data object FormErrors : CustomVpnSideEffect

    data object ConfigurationSaved : CustomVpnSideEffect

    data class FailedToSave(val reason: String) : CustomVpnSideEffect
}
