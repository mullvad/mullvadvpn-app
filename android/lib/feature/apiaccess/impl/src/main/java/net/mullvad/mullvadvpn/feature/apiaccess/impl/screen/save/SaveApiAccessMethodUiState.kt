package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.save

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
