package net.mullvad.mullvadvpn.feature.login.impl.apiunreachable

import net.mullvad.mullvadvpn.feature.login.api.LoginAction

data class ApiUnreachableUiState(
    val showEnableAllAccessMethodsButton: Boolean,
    val noEmailAppAvailable: Boolean,
    val loginAction: LoginAction,
)
