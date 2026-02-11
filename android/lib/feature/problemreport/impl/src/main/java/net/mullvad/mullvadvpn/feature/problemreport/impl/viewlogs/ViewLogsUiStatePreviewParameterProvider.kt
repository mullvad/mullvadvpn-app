package net.mullvad.mullvadvpn.feature.problemreport.impl.viewlogs

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc

private const val SIZE = 50

class ViewLogsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, ViewLogsUiState>> {

    val text = List(SIZE) { "Lorem ipsum dolor ".repeat(SIZE) }

    override val values = sequenceOf(Lc.Content(ViewLogsUiState(text)), Lc.Loading(Unit))
}
