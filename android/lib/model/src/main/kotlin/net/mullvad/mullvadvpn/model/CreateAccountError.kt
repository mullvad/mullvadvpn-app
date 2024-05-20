package net.mullvad.mullvadvpn.model

sealed class CreateAccountError {
    data class Unknown(val error: Throwable) : CreateAccountError()
}
