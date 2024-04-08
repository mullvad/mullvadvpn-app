package net.mullvad.mullvadvpn.model

sealed interface ClearAllOverridesError {
    data class Unknown(val throwable: Throwable) : ClearAllOverridesError
}
