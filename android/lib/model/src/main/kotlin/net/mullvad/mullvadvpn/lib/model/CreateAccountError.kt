package net.mullvad.mullvadvpn.lib.model

sealed class CreateAccountError {
    data class Unknown(val error: Throwable) : CreateAccountError()
}
