package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListItem.CustomListHeader
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId

// Creates a relay list to be displayed by RelayListContent
// Search input as null is defined as not searching
fun relayListItems(
    searchTerm: String? = null,
    relayCountries: List<RelayItem.Location.Country>,
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItemId?,
    expandedItems: Set<String>,
) {
    buildList {
        val relayItems =
            createRelayListItems(searchTerm != null, selectedItem, customLists, relayCountries) {
                it in expandedItems
            }
        if (relayItems.isEmpty() && searchTerm != null) {
            add(RelayListItem.LocationsEmptyText(searchTerm))
        } else {
            addAll(relayItems)
        }
    }
}

private fun createRelayListItems(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    createCustomListSection(isSearching, selectedItem, customLists, isExpanded) +
        createLocationSection(isSearching, selectedItem, countries, isExpanded)

private fun createCustomListSection(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && customLists.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(CustomListHeader)
        val customListItems = createCustomListRelayItems(customLists, selectedItem, isExpanded)
        addAll(customListItems)
        add(RelayListItem.CustomListFooter(customListItems.isNotEmpty()))
    }
}

private fun createCustomListRelayItems(
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItemId?,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    customLists.flatMap { customList ->
        val expanded = isExpanded(customList.id.expandKey())
        buildList {
            add(
                RelayListItem.CustomListItem(
                    customList,
                    isSelected = selectedItem == customList.id,
                    expanded,
                )
            )

            if (expanded) {
                addAll(
                    customList.locations.flatMap {
                        createCustomListEntry(parent = customList, item = it, 1, isExpanded)
                    }
                )
            }
        }
    }

private fun createLocationSection(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && countries.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(RelayListItem.LocationHeader)
        addAll(
            countries.flatMap { country ->
                createGeoLocationEntry(country, selectedItem, isExpanded = isExpanded)
            }
        )
    }
}

private fun createCustomListEntry(
    parent: RelayItem.CustomList,
    item: RelayItem.Location,
    depth: Int = 1,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.CustomListEntryItem> = buildList {
    val expanded = isExpanded(item.id.expandKey(parent.id))
    add(
        RelayListItem.CustomListEntryItem(
            parentId = parent.id,
            parentName = parent.customList.name,
            item = item,
            expanded = expanded,
            depth,
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMap { createCustomListEntry(parent, it, depth + 1, isExpanded) }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap { createCustomListEntry(parent, it, depth + 1, isExpanded) }
                )
            is RelayItem.Location.Relay -> {} // No children to add
        }
    }
}

private fun createGeoLocationEntry(
    item: RelayItem.Location,
    selectedItem: RelayItemId?,
    depth: Int = 0,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.GeoLocationItem> = buildList {
    val expanded = isExpanded(item.id.expandKey())

    add(
        RelayListItem.GeoLocationItem(
            item = item,
            isSelected = selectedItem == item.id,
            depth = depth,
            expanded = expanded,
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMap {
                        createGeoLocationEntry(it, selectedItem, depth + 1, isExpanded)
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap {
                        createGeoLocationEntry(it, selectedItem, depth + 1, isExpanded)
                    }
                )
            is RelayItem.Location.Relay -> {} // Do nothing
        }
    }
}
