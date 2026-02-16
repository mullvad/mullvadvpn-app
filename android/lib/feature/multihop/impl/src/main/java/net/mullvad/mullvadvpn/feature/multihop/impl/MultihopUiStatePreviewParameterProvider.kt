package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc

class MultihopUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, MultihopUiState>> {
    override val values: Sequence<Lc<Boolean, MultihopUiState>> =
        sequenceOf(
            Lc.Loading(false),
            Lc.Content(MultihopUiState(enable = true, isModal = false)),
            Lc.Content(MultihopUiState(enable = false, isModal = true)),
        )
}
