package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId

private val PROVIDER_TO_OWNERSHIPS = mapOf(ProviderId("provider1") to setOf(Ownership.MullvadOwned))

class FilterUiStatePreviewParameterProvider : PreviewParameterProvider<RelayFilterUiState> {
    override val values =
        sequenceOf(
            RelayFilterUiState(
                providerToOwnerships = PROVIDER_TO_OWNERSHIPS,
                selectedOwnership = Ownership.MullvadOwned,
                selectedProviders = PROVIDER_TO_OWNERSHIPS.keys.toList(),
            )
        )
}
