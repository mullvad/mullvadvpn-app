package net.mullvad.mullvadvpn.lib.model

sealed interface ClearAllOverridesError {
    data class Unknown(val throwable: Throwable) : ClearAllOverridesError
}
