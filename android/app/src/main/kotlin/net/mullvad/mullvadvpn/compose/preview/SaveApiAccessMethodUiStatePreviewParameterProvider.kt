package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SaveApiAccessMethodUiState

class SaveApiAccessMethodUiStatePreviewParameterProvider :
    PreviewParameterProvider<SaveApiAccessMethodUiState> {
    override val values: Sequence<SaveApiAccessMethodUiState> =
        sequenceOf(
            SaveApiAccessMethodUiState.Testing,
            SaveApiAccessMethodUiState.SavingAfterSuccessful,
            SaveApiAccessMethodUiState.TestingFailed,
            SaveApiAccessMethodUiState.SavingAfterFailure
        )
}
