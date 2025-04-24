package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.RelayListItemState
import net.mullvad.mullvadvpn.compose.state.RelayListType
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm

// Creates a relay list to be displayed by RelayListContent
internal fun relayListItems(
    searchTerm: String = "",
    isSearching: Boolean = false,
    relayListType: RelayListType,
    relayCountries: List<RelayItem.Location.Country>,
    customLists: List<RelayItem.CustomList>,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    expandedItems: Set<String>,
): List<RelayListItem> {
    val filteredCustomLists = customLists.filterOnSearchTerm(searchTerm)

    return createRelayListItems(
            isSearching = isSearching,
            relayListType = relayListType,
            selectedByThisEntryExitList = selectedByThisEntryExitList,
            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
            customLists = filteredCustomLists,
            countries = relayCountries,
        ) {
            it in expandedItems
        }
        .ifEmpty {
            // Even though we are searching we want to show no locations found text if the
            // search term is empty since we should get a result as long as there are any
            // location in the list
            listOf(
                RelayListItem.LocationsEmptyText(searchTerm, isSearching && searchTerm.isNotEmpty())
            )
        }
}

private fun createRelayListItems(
    isSearching: Boolean,
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    createCustomListSection(
        isSearching,
        relayListType,
        selectedByThisEntryExitList,
        selectedByOtherEntryExitList,
        customLists,
        isExpanded,
    ) +
        createLocationSection(
            isSearching,
            selectedByThisEntryExitList,
            relayListType,
            selectedByOtherEntryExitList,
            countries,
            isExpanded,
        )

private fun createCustomListSection(
    isSearching: Boolean,
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && customLists.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(RelayListItem.CustomListHeader)
        val customListItems =
            createCustomListRelayItems(
                customLists,
                relayListType,
                selectedByThisEntryExitList,
                selectedByOtherEntryExitList,
                isExpanded,
            )
        addAll(customListItems)
        // Do not show the footer in the search view
        if (!isSearching) {
            add(RelayListItem.CustomListFooter(customListItems.isNotEmpty()))
        }
    }
}

private fun createCustomListRelayItems(
    customLists: List<RelayItem.CustomList>,
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    customLists.flatMap { customList ->
        val expanded = isExpanded(customList.id.expandKey())
        buildList {
            add(
                RelayListItem.CustomListItem(
                    item = customList,
                    isSelected = selectedByThisEntryExitList == customList.id,
                    state =
                        customList.createState(
                            relayListType = relayListType,
                            selectedByOtherId = selectedByOtherEntryExitList,
                        ),
                    expanded = expanded,
                )
            )

            if (expanded) {
                addAll(
                    customList.locations.flatMap {
                        createCustomListEntry(
                            parent = customList,
                            item = it,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
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
    selectedByThisEntryExitList: RelayItemId?,
    relayListType: RelayListType,
    selectedByOtherEntryExitList: RelayItemId?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (isSearching && countries.isEmpty()) {
        // If we are searching and no results are found don't show header or footer
    } else {
        add(RelayListItem.LocationHeader)
        addAll(
            countries.flatMap { country ->
                createGeoLocationEntry(
                    item = country,
                    selectedByThisEntryExitList = selectedByThisEntryExitList,
                    relayListType = relayListType,
                    selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                    isExpanded = isExpanded,
                )
            }
        )
    }
}

private fun createCustomListEntry(
    parent: RelayItem.CustomList,
    item: RelayItem.Location,
    relayListType: RelayListType,
    selectedByOtherEntryExitList: RelayItemId?,
    depth: Int = 1,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.CustomListEntryItem> = buildList {
    val expanded = isExpanded(item.id.expandKey(parent.id))
    add(
        RelayListItem.CustomListEntryItem(
            parentId = parent.id,
            parentName = parent.customList.name,
            item = item,
            state =
                item.createState(
                    relayListType = relayListType,
                    selectedByOtherId = selectedByOtherEntryExitList,
                ),
            expanded = expanded,
            depth = depth,
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMap {
                        createCustomListEntry(
                            parent = parent,
                            item = it,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                        )
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap {
                        createCustomListEntry(
                            parent = parent,
                            item = it,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                        )
                    }
                )
            is RelayItem.Location.Relay -> {} // No children to add
        }
    }
}

private fun createGeoLocationEntry(
    item: RelayItem.Location,
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    depth: Int = 0,
    isExpanded: (String) -> Boolean,
): List<RelayListItem.GeoLocationItem> = buildList {
    val expanded = isExpanded(item.id.expandKey())

    add(
        RelayListItem.GeoLocationItem(
            item = item,
            isSelected = selectedByThisEntryExitList == item.id,
            state =
                item.createState(
                    relayListType = relayListType,
                    selectedByOtherId = selectedByOtherEntryExitList,
                ),
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
                            item = it,
                            relayListType = relayListType,
                            selectedByThisEntryExitList = selectedByThisEntryExitList,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                        )
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMap {
                        createGeoLocationEntry(
                            item = it,
                            relayListType = relayListType,
                            selectedByThisEntryExitList = selectedByThisEntryExitList,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
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

internal fun RelayItemSelection.selectedByThisEntryExitList(relayListType: RelayListType) =
    when (this) {
        is RelayItemSelection.Multiple ->
            when (relayListType) {
                RelayListType.ENTRY -> entryLocation
                RelayListType.EXIT -> exitLocation
            }.getOrNull()
        is RelayItemSelection.Single -> exitLocation.getOrNull()
    }

internal fun RelayItemSelection.selectedByOtherEntryExitList(
    relayListType: RelayListType,
    customLists: List<RelayItem.CustomList>,
) =
    when (this) {
        is RelayItemSelection.Multiple -> {
            val location =
                when (relayListType) {
                    RelayListType.ENTRY -> exitLocation
                    RelayListType.EXIT -> entryLocation
                }.getOrNull()
            location.singleRelayId(customLists)
        }
        is RelayItemSelection.Single -> null
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

private fun RelayItem.createState(
    relayListType: RelayListType,
    selectedByOtherId: RelayItemId?,
): RelayListItemState? {
    val isSelectedByOther =
        when (this) {
            is RelayItem.CustomList -> {
                selectedByOtherId == customList.id ||
                    (customList.locations.isNotEmpty() &&
                        customList.locations.all { it == selectedByOtherId })
            }
            is RelayItem.Location.City -> selectedByOtherId == id
            is RelayItem.Location.Country -> selectedByOtherId == id
            is RelayItem.Location.Relay -> selectedByOtherId == id
        }
    return if (isSelectedByOther) {
        when (relayListType) {
            RelayListType.ENTRY -> RelayListItemState.USED_AS_EXIT
            RelayListType.EXIT -> RelayListItemState.USED_AS_ENTRY
        }
    } else {
        null
    }
}
