package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.ModelOwnership

class SelectLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<SelectLocationUiState> {
    override val values =
        sequenceOf(
            SelectLocationUiState.Loading,
            SelectLocationUiState.Data(
                filterChips = emptyList(),
                multihopEnabled = false,
                relayListType = RelayListType.EXIT,
            ),
            SelectLocationUiState.Data(
                filterChips =
                    listOf(
                        FilterChip.Ownership(ownership = ModelOwnership.Rented),
                        FilterChip.Provider(PROVIDER_COUNT),
                    ),
                multihopEnabled = false,
                relayListType = RelayListType.EXIT,
            ),
            SelectLocationUiState.Data(
                filterChips = emptyList(),
                multihopEnabled = true,
                relayListType = RelayListType.ENTRY,
            ),
            SelectLocationUiState.Data(
                filterChips =
                    listOf(
                        FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned),
                        FilterChip.Provider(PROVIDER_COUNT),
                    ),
                multihopEnabled = true,
                relayListType = RelayListType.ENTRY,
            ),
        )
}

private const val PROVIDER_COUNT = 3
