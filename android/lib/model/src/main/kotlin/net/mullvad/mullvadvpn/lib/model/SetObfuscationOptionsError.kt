package net.mullvad.mullvadvpn.lib.model

sealed interface SetObfuscationOptionsError {
    data class Unknown(val throwable: Throwable) : SetObfuscationOptionsError
}
