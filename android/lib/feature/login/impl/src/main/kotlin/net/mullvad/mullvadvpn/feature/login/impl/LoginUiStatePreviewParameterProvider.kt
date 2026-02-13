package net.mullvad.mullvadvpn.feature.login.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

class LoginUiStatePreviewParameterProvider : PreviewParameterProvider<LoginUiState> {
    override val values: Sequence<LoginUiState>
        get() =
            sequenceOf(
                LoginUiState(),
                LoginUiState(loginState = LoginState.Loading.LoggingIn),
                LoginUiState(loginState = LoginState.Loading.CreatingAccount),
                LoginUiState(
                    loginState = LoginState.Idle(LoginUiStateError.LoginError.InvalidCredentials)
                ),
                LoginUiState(loginState = LoginState.Success),
            )
}
