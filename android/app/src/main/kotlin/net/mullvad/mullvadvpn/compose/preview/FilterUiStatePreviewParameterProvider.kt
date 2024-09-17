package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId

val PROVIDER = Provider(providerId = ProviderId("provider1"), ownership = Ownership.MullvadOwned)

class FilterUiStatePreviewParameterProvider : PreviewParameterProvider<RelayFilterUiState> {
    override val values =
        sequenceOf(
            RelayFilterUiState(
                selectedOwnership = Ownership.MullvadOwned,
                allProviders = listOf(PROVIDER),
                selectedProviders = listOf(PROVIDER),
            )
        )
}
