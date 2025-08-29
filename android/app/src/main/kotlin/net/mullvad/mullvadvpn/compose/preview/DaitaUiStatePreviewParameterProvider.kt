package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.DaitaUiState
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class DaitaUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Boolean, DaitaUiState>> {
    override val values: Sequence<Lc<Boolean, DaitaUiState>> =
        sequenceOf(
            Lc.Loading(true),
            DaitaUiState(daitaEnabled = true, directOnly = false, isModal = false).toLc(),
            DaitaUiState(daitaEnabled = true, directOnly = true, isModal = true).toLc(),
        )
}
