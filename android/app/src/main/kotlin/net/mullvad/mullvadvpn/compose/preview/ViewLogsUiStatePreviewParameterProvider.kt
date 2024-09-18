package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState

private const val SIZE = 50

class ViewLogsUiStatePreviewParameterProvider : PreviewParameterProvider<ViewLogsUiState> {

    val text = List(SIZE) { "Lorem ipsum dolor ".repeat(SIZE) }

    override val values =
        sequenceOf(ViewLogsUiState(text, isLoading = false), ViewLogsUiState(isLoading = true))
}
