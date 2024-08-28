package net.mullvad.mullvadvpn.compose.state

data class SaveApiAccessMethodUiState(
    val testingState: TestApiAccessMethodState = TestApiAccessMethodState.Testing,
    val isSaving: Boolean = false,
)

sealed interface TestApiAccessMethodState {
    data object Testing : TestApiAccessMethodState

    sealed interface Result : TestApiAccessMethodState {
        data object Successful : Result

        data object Failure : Result
    }
}
