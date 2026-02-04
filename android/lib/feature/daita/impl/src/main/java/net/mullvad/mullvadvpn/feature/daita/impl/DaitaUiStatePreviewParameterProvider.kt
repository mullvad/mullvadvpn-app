package net.mullvad.mullvadvpn.feature.daita.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.toLc

class DaitaUiStatePreviewParameterProvider : PreviewParameterProvider<Lc<Boolean, DaitaUiState>> {
    override val values: Sequence<Lc<Boolean, DaitaUiState>> =
        sequenceOf(
            Lc.Loading(true),
            DaitaUiState(daitaEnabled = true, directOnly = false, isModal = false).toLc(),
            DaitaUiState(daitaEnabled = true, directOnly = true, isModal = true).toLc(),
        )
}
