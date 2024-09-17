package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.FilterChip
import net.mullvad.mullvadvpn.compose.state.ModelOwnership
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.DomainCustomList
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.RelayItem

val RELAY =
    RelayItem.Location.Relay(
        id =
            GeoLocationId.Hostname(
                city = GeoLocationId.City(country = GeoLocationId.Country("se"), code = "code"),
                code = "code",
            ),
        provider = Provider(providerId = ProviderId("providerId"), ownership = Ownership.Rented),
        active = true,
        daita = true,
    )

class SelectLocationsUiStatePreviewParameterProvider :
    PreviewParameterProvider<SelectLocationUiState> {
    override val values =
        sequenceOf(
            SelectLocationUiState.Content(
                searchTerm = "search term",
                listOf(FilterChip.Ownership(ownership = ModelOwnership.MullvadOwned)),
                relayListItems =
                    listOf(
                        RelayListItem.GeoLocationItem(
                            item = RELAY,
                            isSelected = true,
                            depth = 1,
                            expanded = true,
                        )
                    ),
                customLists =
                    listOf(
                        RelayItem.CustomList(
                            customList =
                                DomainCustomList(
                                    id = CustomListId("custom_list_id"),
                                    locations =
                                        listOf(
                                            GeoLocationId.City(
                                                country = GeoLocationId.Country("dk"),
                                                code = "code2",
                                            )
                                        ),
                                    name = CustomListName.fromString("Custom List"),
                                ),
                            locations = listOf(RELAY),
                        )
                    ),
            ),
            SelectLocationUiState.Loading,
        )
}
