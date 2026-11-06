package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.edit

sealed interface PersonalVpnSideEffect {

    data object FormErrors : PersonalVpnSideEffect

    data object ConfigurationSaved : PersonalVpnSideEffect

    data class FailedToSave(val reason: String) : PersonalVpnSideEffect
}
