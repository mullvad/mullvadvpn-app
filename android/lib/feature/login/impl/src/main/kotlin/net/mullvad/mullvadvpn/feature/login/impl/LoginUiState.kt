package net.mullvad.mullvadvpn.feature.login.impl

import net.mullvad.mullvadvpn.lib.model.AccountNumber

data class LoginUiState(
    val accountNumberInput: String = "",
    val lastUsedAccount: AccountNumber? = null,
    val loginState: LoginState = LoginState.Idle(null),
) {
    val loginButtonEnabled = loginState is LoginState.Idle

    companion object {
        val INITIAL = LoginUiState()
    }
}

sealed interface LoginState {
    fun isError() = this is Idle && loginUiStateError != null

    data class Idle(val loginUiStateError: LoginUiStateError? = null) : LoginState

    sealed interface Loading : LoginState {
        data object LoggingIn : Loading

        data object CreatingAccount : Loading
    }

    data object Success : LoginState
}

sealed interface LoginUiStateError {
    sealed interface LoginError : LoginUiStateError {
        data object Empty : LoginError

        data object InvalidCredentials : LoginError

        sealed class InvalidInput : LoginError {
            abstract val accountNumber: AccountNumber

            data class TooShort(override val accountNumber: AccountNumber) : InvalidInput()

            data class TooLong(override val accountNumber: AccountNumber) : InvalidInput()
        }

        data object TooManyAttempts : LoginError

        data object ApiUnreachable : LoginError

        data class Unknown(val reason: String) : LoginError

        data object NoInternetConnection : LoginError
    }
}
