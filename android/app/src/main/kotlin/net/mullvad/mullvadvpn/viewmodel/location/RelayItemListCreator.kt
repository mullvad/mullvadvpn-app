package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListSelection
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.SelectedLocation
import net.mullvad.mullvadvpn.relaylist.MIN_SEARCH_LENGTH
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm

// Creates a relay list to be displayed by RelayListContent
internal fun relayListItems(
    searchTerm: String = "",
    relayCountries: List<RelayItem.Location.Country>,
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    expandedItems: Set<String>,
): List<RelayListItem> {
    val filteredCustomLists = customLists.filterOnSearchTerm(searchTerm)

    return buildList {
        val relayItems =
            createRelayListItems(
                isSearching = searchTerm.isSearching(),
                selectedItem = selectedItem,
                disabledItem = disabledItem,
                customLists = filteredCustomLists,
                countries = relayCountries,
            ) {
                it in expandedItems
            }
        if (relayItems.isEmpty()) {
            add(RelayListItem.LocationsEmptyText(searchTerm))
        } else {
            addAll(relayItems)
        }
    }
}

private fun createRelayListItems(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    createCustomListSection(isSearching, selectedItem, disabledItem, customLists, isExpanded) +
        createLocationSection(isSearching, selectedItem, disabledItem, countries, isExpanded)

private fun createCustomListSection(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && customLists.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(RelayListItem.CustomListHeader)
        val customListItems =
            createCustomListRelayItems(customLists, selectedItem, disabledItem, isExpanded)
        addAll(customListItems)
        // Do not show the footer in the search view
        if (!isSearching) {
            add(RelayListItem.CustomListFooter(customListItems.isNotEmpty()))
        }
    }
}

private fun createCustomListRelayItems(
    customLists: List<RelayItem.CustomList>,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    customLists.flatMap { customList ->
        val expanded = isExpanded(customList.id.expandKey())
        buildList {
            add(
                RelayListItem.CustomListItem(
                    item = customList,
                    isSelected = selectedItem == customList.id,
                    isEnabled =
                        disabledItem != customList.id &&
                            customList.locations.singleOrNull()?.id?.let { it != disabledItem } !=
                                false,
                    expanded = expanded,
                )
            )

            if (expanded) {
                addAll(
                    customList.locations.flatMap {
                        createCustomListEntry(
                            parent = customList,
                            item = it,
                            disabledItem = disabledItem,
                            depth = 1,
                            isExpanded = isExpanded,
                        )
                    }
                )
            }
        }
    }

private fun createLocationSection(
    isSearching: Boolean,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && countries.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(RelayListItem.LocationHeader)
        addAll(
            countries.flatMap { country ->
                createGeoLocationEntry(country, selectedItem, disabledItem, isExpanded = isExpanded)
            }
        )
    }
}

private fun createCustomListEntry(
    parent: RelayItem.CustomList,
    item: RelayItem.Location,
    disabledItem: RelayItemId?,
    depth: Int = 1,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.CustomListEntryItem> = buildList {
    val expanded = isExpanded(item.id.expandKey(parent.id))
    add(
        RelayListItem.CustomListEntryItem(
            parentId = parent.id,
            parentName = parent.customList.name,
            item = item,
            isEnabled = item.id != disabledItem,
            expanded = expanded,
            depth = depth,
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMap {
                        createCustomListEntry(parent, it, disabledItem, depth + 1, isExpanded)
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap {
                        createCustomListEntry(parent, it, disabledItem, depth + 1, isExpanded)
                    }
                )
            is RelayItem.Location.Relay -> {} // No children to add
        }
    }
}

private fun createGeoLocationEntry(
    item: RelayItem.Location,
    selectedItem: RelayItemId?,
    disabledItem: RelayItemId?,
    depth: Int = 0,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.GeoLocationItem> = buildList {
    val expanded = isExpanded(item.id.expandKey())

    add(
        RelayListItem.GeoLocationItem(
            item = item,
            isSelected = selectedItem == item.id,
            isEnabled = disabledItem != item.id,
            depth = depth,
            expanded = expanded,
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMap {
                        createGeoLocationEntry(
                            it,
                            selectedItem,
                            disabledItem,
                            depth + 1,
                            isExpanded,
                        )
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap {
                        createGeoLocationEntry(
                            it,
                            selectedItem,
                            disabledItem,
                            depth + 1,
                            isExpanded,
                        )
                    }
                )
            is RelayItem.Location.Relay -> {} // Do nothing
        }
    }
}

internal fun RelayItemId.expandKey(parent: CustomListId? = null) =
    (parent?.value ?: "") +
        when (this) {
            is CustomListId -> value
            is GeoLocationId -> code
        }

internal fun SelectedLocation.getForRelayListSelect(relayListSelection: RelayListSelection) =
    when (this) {
        is SelectedLocation.Multiple ->
            when (relayListSelection) {
                RelayListSelection.Entry -> entryLocation
                RelayListSelection.Exit -> exitLocation
            }.getOrNull()
        is SelectedLocation.Single -> exitLocation.getOrNull()
    }

internal fun SelectedLocation.getForRelayListDisabled(
    relayListSelection: RelayListSelection,
    customLists: List<RelayItem.CustomList>,
) =
    when (this) {
        is SelectedLocation.Multiple -> {
            val location =
                when (relayListSelection) {
                    RelayListSelection.Entry -> exitLocation
                    RelayListSelection.Exit -> entryLocation
                }.getOrNull()
            location.singleRelayId(customLists)
        }
        is SelectedLocation.Single -> null
    }

// We only want to block selecting the same entry as exit if it is a relay. For country and
// city it is fine to have same entry and exit
// For custom lists we will block if the custom lists only contains one relay and
// nothing else
private fun RelayItemId?.singleRelayId(customLists: List<RelayItem.CustomList>): RelayItemId? =
    when (this) {
        is GeoLocationId.City,
        is GeoLocationId.Country -> null
        is GeoLocationId.Hostname -> this
        is CustomListId ->
            customLists
                .firstOrNull { customList -> customList.id == this }
                ?.locations
                ?.singleOrNull()
                ?.id as? GeoLocationId.Hostname
        else -> null
    }

private fun String.isSearching() = length >= MIN_SEARCH_LENGTH
