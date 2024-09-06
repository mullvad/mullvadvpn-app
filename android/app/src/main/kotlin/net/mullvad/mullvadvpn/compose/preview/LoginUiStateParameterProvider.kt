package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginUiState

class LoginUiStateParameterProvider : PreviewParameterProvider<LoginUiState> {
    override val values: Sequence<LoginUiState>
        get() =
            sequenceOf(
                LoginUiState(),
                LoginUiState(loginState = LoginState.Loading.LoggingIn),
                LoginUiState(loginState = LoginState.Loading.CreatingAccount),
                LoginUiState(loginState = LoginState.Idle(LoginError.InvalidCredentials)),
                LoginUiState(loginState = LoginState.Success),
            )
}
