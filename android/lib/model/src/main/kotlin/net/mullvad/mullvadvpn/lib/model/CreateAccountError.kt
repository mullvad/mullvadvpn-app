package net.mullvad.mullvadvpn.lib.model

sealed class CreateAccountError {
    data object TooManyAttempts : CreateAccountError()

    data object ApiUnreachable : CreateAccountError()

    data object TimeOut : CreateAccountError()

    data class Unknown(val error: Throwable) : CreateAccountError()
}
