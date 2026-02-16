package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import net.mullvad.mullvadvpn.lib.model.CustomList
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

object RelayListItemPreviewData {
    @Suppress("LongMethod")
    fun generateRelayListItems(
        includeCustomLists: Boolean,
        isSearching: Boolean,
    ): List<RelayListItem> = buildList {
        if (!isSearching || includeCustomLists) {
            add(RelayListItem.CustomListHeader)
            // Add custom list items
            if (includeCustomLists) {
                RelayListItem.CustomListItem(
                    item =
                        RelayItem.CustomList(
                            customList =
                                CustomList(
                                    id = CustomListId("custom_list_id"),
                                    name = CustomListName.fromString("Custom List"),
                                    locations = emptyList(),
                                ),
                            locations =
                                listOf(
                                    generateRelayItemCountry(
                                        name = "Country",
                                        cityNames = listOf("City"),
                                        relaysPerCity = 2,
                                        active = true,
                                    )
                                ),
                        ),
                    isSelected = false,
                    state = null,
                    expanded = false,
                    itemPosition = Position.Single,
                )
            }
            if (!isSearching) {
                add(RelayListItem.CustomListFooter(hasCustomList = includeCustomLists))
            }
        }
        add(RelayListItem.LocationHeader)
        val locations =
            listOf(
                generateRelayItemCountry(
                    name = "First Country",
                    cityNames = listOf("Capital City", "Minor City"),
                    relaysPerCity = 2,
                    active = true,
                ),
                generateRelayItemCountry(
                    name = "Second Country",
                    cityNames = listOf("Medium City", "Small City", "Vivec City"),
                    relaysPerCity = 1,
                    active = false,
                ),
            )
        addAll(
            listOf(
                RelayListItem.GeoLocationItem(
                    item = locations[0],
                    isSelected = false,
                    hierarchy = Hierarchy.Parent,
                    expanded = true,
                    state = null,
                    itemPosition = Position.Middle,
                ),
                RelayListItem.GeoLocationItem(
                    item = locations[0].cities[0],
                    isSelected = true,
                    hierarchy = Hierarchy.Child1,
                    expanded = false,
                    state = null,
                    itemPosition = Position.Middle,
                ),
                RelayListItem.GeoLocationItem(
                    item = locations[0].cities[1],
                    isSelected = false,
                    hierarchy = Hierarchy.Child1,
                    expanded = true,
                    state = null,
                    itemPosition = Position.Middle,
                ),
                RelayListItem.GeoLocationItem(
                    item = locations[0].cities[1].relays[0],
                    isSelected = false,
                    hierarchy = Hierarchy.Child2,
                    expanded = false,
                    state = RelayListItemState.USED_AS_EXIT,
                    itemPosition = Position.Middle,
                ),
                RelayListItem.GeoLocationItem(
                    item = locations[0].cities[1].relays[1],
                    isSelected = false,
                    hierarchy = Hierarchy.Child2,
                    expanded = false,
                    state = null,
                    itemPosition = Position.Middle,
                ),
                RelayListItem.GeoLocationItem(
                    item = locations[1],
                    isSelected = false,
                    hierarchy = Hierarchy.Parent,
                    expanded = false,
                    state = null,
                    itemPosition = Position.Bottom,
                ),
            )
        )
    }

    fun generateEmptyList(searchTerm: String, isSearching: Boolean) =
        listOf(
            if (isSearching) {
                RelayListItem.LocationsEmptyText(searchTerm)
            } else {
                RelayListItem.EmptyRelayList
            }
        )
}
