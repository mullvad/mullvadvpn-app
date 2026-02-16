package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.save

import androidx.compose.ui.tooling.preview.PreviewParameterProvider

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
