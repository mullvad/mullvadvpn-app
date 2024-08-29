package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState
import net.mullvad.mullvadvpn.compose.state.TestApiAccessMethodState

class SaveApiAccessMethodUiStatePreviewParameterProvider :
    PreviewParameterProvider<SaveApiAccessMethodUiState> {
    override val values: Sequence<SaveApiAccessMethodUiState> =
        sequenceOf(
            SaveApiAccessMethodUiState(testingState = TestApiAccessMethodState.Testing),
            SaveApiAccessMethodUiState(
                testingState = TestApiAccessMethodState.Result.Successful,
                isSaving = true,
            ),
            SaveApiAccessMethodUiState(
                testingState = TestApiAccessMethodState.Result.Failure,
                isSaving = false,
            ),
            SaveApiAccessMethodUiState(
                testingState = TestApiAccessMethodState.Result.Failure,
                isSaving = true,
            ),
        )
}
