package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.AccountToken

data class LoginUiState(
    val lastUsedAccount: AccountToken? = null,
    val loginState: LoginState = LoginState.Idle(null)
) {
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
}
