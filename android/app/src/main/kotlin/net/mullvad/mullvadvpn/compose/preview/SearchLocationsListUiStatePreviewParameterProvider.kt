package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.MultihopRelayListType
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.compose.state.SelectLocationListUiState
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemPreviewData
import net.mullvad.mullvadvpn.util.Lce

class SearchLocationsListUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<Unit, SelectLocationListUiState, Unit>> {
    override val values =
        sequenceOf(
            Lce.Content(
                SelectLocationListUiState(
                    relayListItems =
                        RelayListItemPreviewData.generateRelayListItems(
                            includeCustomLists = true,
                            isSearching = false,
                        ),
                    customLists = emptyList(),
                    relayListType = RelayListType.Multihop(MultihopRelayListType.EXIT),
                    selection = RelayItemSelection.Single(Constraint.Any),
                )
            ),
            Lce.Loading(Unit),
            Lce.Error(Unit),
        )
}
