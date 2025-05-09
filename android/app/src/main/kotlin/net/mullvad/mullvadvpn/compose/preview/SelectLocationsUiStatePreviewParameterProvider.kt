package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.usecase.FilterChip
import net.mullvad.mullvadvpn.usecase.ModelOwnership
import net.mullvad.mullvadvpn.util.Lc

class SelectLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lc<Unit, SelectLocationUiState>> {
    override val values =
        sequenceOf(
            Lc.Loading(Unit),
            Lc.Content(
                SelectLocationUiState(
                    filterChips = emptyList(),
                    multihopEnabled = false,
                    relayListType = RelayListType.EXIT,
                    enableTopBar = true,
                )
            ),
            Lc.Content(
                SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.Rented),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = false,
                    relayListType = RelayListType.EXIT,
                    enableTopBar = true,
                )
            ),
            Lc.Content(
                SelectLocationUiState(
                    filterChips = emptyList(),
                    multihopEnabled = true,
                    relayListType = RelayListType.ENTRY,
                    enableTopBar = true,
                )
            ),
            Lc.Content(
                SelectLocationUiState(
                    filterChips =
                        listOf(
                            FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned),
                            FilterChip.Provider(PROVIDER_COUNT),
                        ),
                    multihopEnabled = true,
                    relayListType = RelayListType.ENTRY,
                    enableTopBar = true,
                )
            ),
        )
}

private const val PROVIDER_COUNT = 3
