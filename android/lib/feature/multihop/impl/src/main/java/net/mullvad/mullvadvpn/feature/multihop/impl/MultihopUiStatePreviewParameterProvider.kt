package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.MultihopMode

class MultihopUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Boolean, MultihopUiState>> {
    override val values: Sequence<Lc<Boolean, MultihopUiState>> =
        sequenceOf(
            Lc.Loading(false),
            Lc.Content(MultihopUiState(mode = MultihopMode.ALWAYS, isModal = false)),
            Lc.Content(MultihopUiState(mode = MultihopMode.WHEN_NEEDED, isModal = false)),
            Lc.Content(MultihopUiState(mode = MultihopMode.NEVER, isModal = true)),
        )
}
