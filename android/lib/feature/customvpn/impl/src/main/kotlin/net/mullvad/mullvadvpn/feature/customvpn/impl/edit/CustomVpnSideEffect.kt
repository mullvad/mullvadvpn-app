package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

sealed interface CustomVpnSideEffect {

    enum class FormErrors : CustomVpnSideEffect {
        FieldError,
        MissingPeerError,
    }

    data object ConfigurationSaved : CustomVpnSideEffect

    data object ConfigurationDeleted : CustomVpnSideEffect

    data class FailedToSave(val reason: String) : CustomVpnSideEffect
}
