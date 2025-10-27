package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
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
                    filterChips = emptyMap(),
                    multihopEnabled = false,
                    relayListType = RelayListType.Single,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    entrySelection = null,
                    exitSelection = null,
                    tunnelErrorStateCause = null,
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
                    multihopEnabled = false,
                    relayListType = RelayListType.Single,
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    entrySelection = null,
                    exitSelection = null,
                    tunnelErrorStateCause = null,
                )
                .toLc(),
            SelectLocationUiState(
                    filterChips = emptyMap(),
                    multihopEnabled = true,
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    entrySelection = null,
                    exitSelection = null,
                    tunnelErrorStateCause = null,
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
                    multihopEnabled = true,
                    relayListType = RelayListType.Multihop(MultihopRelayListType.ENTRY),
                    isSearchButtonEnabled = true,
                    isFilterButtonEnabled = true,
                    isRecentsEnabled = true,
                    entrySelection = null,
                    exitSelection = null,
                    tunnelErrorStateCause = null,
                )
                .toLc(),
        )
}

private const val PROVIDER_COUNT = 3
