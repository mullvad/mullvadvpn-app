package net.mullvad.mullvadvpn.compose.state

sealed interface SaveApiAccessMethodUiState {
    data object Testing : SaveApiAccessMethodUiState

    data object TestingFailed : SaveApiAccessMethodUiState

    data object SavingAfterSuccessful : SaveApiAccessMethodUiState

    data object SavingAfterFailure : SaveApiAccessMethodUiState
}
