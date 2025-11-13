package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.HopSelection
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
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
                    multihopListSelection = MultihopRelayListType.EXIT,
                    filterChips = emptyMap(),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    hopSelection = HopSelection.Single(null),
                    tunnelErrorStateCause = null,
                    entrySelectionAllowed = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        mapOf(
                            RelayListType.Single to
                                listOf(
                                    FilterChip.Ownership(ownership = ModelOwnership.Rented),
                                    FilterChip.Provider(PROVIDER_COUNT),
                                )
                        ),
                    multihopListSelection = MultihopRelayListType.EXIT,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    hopSelection = HopSelection.Single(null),
                    tunnelErrorStateCause = null,
                    entrySelectionAllowed = true,
                )
                .toLc(),
            SelectLocationUiState(
                    multihopListSelection = MultihopRelayListType.ENTRY,
                    filterChips = emptyMap(),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    hopSelection = HopSelection.Multi(null, null),
                    tunnelErrorStateCause = null,
                    entrySelectionAllowed = true,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips =
                        mapOf(
                            RelayListType.Multihop(MultihopRelayListType.ENTRY) to
                                listOf(
                                    FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned),
                                    FilterChip.Provider(PROVIDER_COUNT),
                                )
                        ),
                    multihopListSelection = MultihopRelayListType.ENTRY,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    hopSelection = HopSelection.Multi(null, null),
                    tunnelErrorStateCause = null,
                    entrySelectionAllowed = true,
                )
                .toLc(),
        )
}

private const val PROVIDER_COUNT = 3
