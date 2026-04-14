package net.mullvad.mullvadvpn.feature.login.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.AccountNumber

class LoginUiStatePreviewParameterProvider : PreviewParameterProvider<LoginUiState> {
    override val values: Sequence<LoginUiState>
        get() =
            sequenceOf(
                LoginUiState(),
                LoginUiState(accountNumberInput = "1234123412341234"),
                LoginUiState(
                    accountNumberInput = "1234123412341234",
                    lastUsedAccount = AccountNumber("4321432143214321"),
                ),
                LoginUiState(loginState = LoginState.Loading.LoggingIn),
                LoginUiState(loginState = LoginState.Loading.CreatingAccount),
                LoginUiState(
                    accountNumberInput = "1234123412341234",
                    loginState = LoginState.Idle(LoginUiStateError.LoginError.InvalidCredentials),
                ),
                LoginUiState(
                    accountNumberInput = "1234123412341234",
                    loginState = LoginState.Idle(LoginUiStateError.LoginError.InvalidCredentials),
                    lastUsedAccount = AccountNumber("4321432143214321"),
                ),
                LoginUiState(loginState = LoginState.Success),
            )
}
