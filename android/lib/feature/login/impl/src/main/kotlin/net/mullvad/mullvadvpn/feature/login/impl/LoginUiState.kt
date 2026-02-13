package net.mullvad.mullvadvpn.feature.login.impl

import net.mullvad.mullvadvpn.lib.model.AccountNumber

const val MIN_ACCOUNT_LOGIN_LENGTH = 8

data class LoginUiState(
    val accountNumberInput: String = "",
    val lastUsedAccount: AccountNumber? = null,
    val loginState: LoginState = LoginState.Idle(null),
) {
    val loginButtonEnabled =
        accountNumberInput.length >= MIN_ACCOUNT_LOGIN_LENGTH && loginState is LoginState.Idle

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
    sealed interface CreateAccountError : LoginUiStateError {
        data object TooManyAttempts : CreateAccountError

        data object ApiUnreachable : CreateAccountError

        data object NoInternetConnection : CreateAccountError

        data object Unknown : CreateAccountError
    }

    sealed interface LoginError : LoginUiStateError {
        data object InvalidCredentials : LoginError

        data class InvalidInput(val accountNumber: AccountNumber) : LoginError

        data object TooManyAttempts : LoginError

        data object ApiUnreachable : LoginError

        data class Unknown(val reason: String) : LoginError

        data object NoInternetConnection : LoginError
    }
}
