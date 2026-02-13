package net.mullvad.mullvadvpn.feature.login.impl.apiunreachable

data class ApiUnreachableUiState(
    val showEnableAllAccessMethodsButton: Boolean,
    val noEmailAppAvailable: Boolean,
    val loginAction: LoginAction,
)
