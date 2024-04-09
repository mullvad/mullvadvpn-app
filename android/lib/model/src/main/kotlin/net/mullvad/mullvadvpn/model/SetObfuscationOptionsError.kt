package net.mullvad.mullvadvpn.model

sealed interface SetObfuscationOptionsError {
    data class Unknown(val throwable: Throwable) : SetObfuscationOptionsError
}
