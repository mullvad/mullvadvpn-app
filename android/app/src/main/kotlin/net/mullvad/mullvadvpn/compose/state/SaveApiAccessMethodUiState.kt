package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.TestApiAccessMethodState

data class SaveApiAccessMethodUiState(
    val testingState: TestApiAccessMethodState = TestApiAccessMethodState.Testing,
    val isSaving: Boolean = false
)
