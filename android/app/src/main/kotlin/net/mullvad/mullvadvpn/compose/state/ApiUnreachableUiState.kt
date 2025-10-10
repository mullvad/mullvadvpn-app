package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.compose.dialog.info.LoginAction

data class ApiUnreachableUiState(
    val showEnableAllAccessMethodsButton: Boolean,
    val loginAction: LoginAction,
)
