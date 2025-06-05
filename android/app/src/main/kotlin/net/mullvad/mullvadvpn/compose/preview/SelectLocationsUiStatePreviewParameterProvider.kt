package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.ModelOwnership
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.toLc

class SelectLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, SelectLocationUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            SelectLocationUiState(
                    filterChips = emptyList(),
                    multihopEnabled = false,
                    relayListType = RelayListType.EXIT,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.Rented),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = false,
                    relayListType = RelayListType.EXIT,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips = emptyList(),
                    multihopEnabled = true,
                    relayListType = RelayListType.ENTRY,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = true,
                    relayListType = RelayListType.ENTRY,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                )
                .toLc(),
        )
}

private const val PROVIDER_COUNT = 3
