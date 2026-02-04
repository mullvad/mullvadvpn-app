package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState

private const val SIZE = 50

class ViewLogsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, ViewLogsUiState>> {

    val text = List(SIZE) { "Lorem ipsum dolor ".repeat(SIZE) }

    override val values = sequenceOf(Lc.Content(ViewLogsUiState(text)), Lc.Loading(Unit))
}
