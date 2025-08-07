package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
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
                    relayListType = RelayListType.Single,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.Rented),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = false,
                    relayListType = RelayListType.Single,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips = emptyList(),
                    multihopEnabled = true,
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = true,
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                )
                .toLc(),
        )
}

private const val PROVIDER_COUNT = 3
