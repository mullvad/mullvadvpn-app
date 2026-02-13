package net.mullvad.mullvadvpn.feature.location.impl.list

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemPreviewData

class SelectLocationsListUiStatePreviewParameterProvider :
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
                )
            ),
            Lce.Loading(Unit),
            Lce.Error(Unit),
        )
}
