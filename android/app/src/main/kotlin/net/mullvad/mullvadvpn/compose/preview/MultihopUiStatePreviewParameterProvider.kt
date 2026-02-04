package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.core.Lc
import net.mullvad.mullvadvpn.viewmodel.MultihopUiState

class MultihopUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, MultihopUiState>> {
    override val values: Sequence<Lc<Boolean, MultihopUiState>> =
        sequenceOf(
            Lc.Loading(false),
            Lc.Content(MultihopUiState(enable = true, isModal = false)),
            Lc.Content(MultihopUiState(enable = false, isModal = true)),
        )
}
