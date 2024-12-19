package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId

private val PROVIDER =
    Provider(providerId = ProviderId("provider1"), ownership = setOf(Ownership.MullvadOwned))

class FilterUiStatePreviewParameterProvider : PreviewParameterProvider<RelayFilterUiState> {
    override val values =
        sequenceOf(
            RelayFilterUiState(
                selectedOwnership = Ownership.MullvadOwned,
                filteredProvidersByOwnership = listOf(PROVIDER.providerId),
                selectedProviders = listOf(PROVIDER.providerId),
            )
        )
}
