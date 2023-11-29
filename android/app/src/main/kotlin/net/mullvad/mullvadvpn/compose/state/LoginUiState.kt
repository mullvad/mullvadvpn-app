package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.AccountToken

const val MIN_ACCOUNT_LOGIN_LENGTH = 8

data class LoginUiState(
    val accountNumberInput: String = "",
    val lastUsedAccount: AccountToken? = null,
    val loginState: LoginState = LoginState.Idle(null)
) {
    val loginButtonEnabled =
        accountNumberInput.length >= MIN_ACCOUNT_LOGIN_LENGTH && loginState is LoginState.Idle

    companion object {
        val INITIAL = LoginUiState()
    }
}

sealed interface LoginState {
    fun isError() = this is Idle && loginError != null

    data class Idle(val loginError: LoginError? = null) : LoginState

    sealed interface Loading : LoginState {
        data object LoggingIn : Loading

        data object CreatingAccount : Loading
    }

    data object Success : LoginState
}

sealed class LoginError {
    data object UnableToCreateAccount : LoginError()

    data object InvalidCredentials : LoginError()

    data class Unknown(val reason: String) : LoginError()

    data object NoInternetConnection : LoginError()
}
