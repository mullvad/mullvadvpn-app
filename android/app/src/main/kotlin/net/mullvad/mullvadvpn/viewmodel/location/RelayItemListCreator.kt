package net.mullvad.mullvadvpn.viewmodel.location

import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.Hop
import net.mullvad.mullvadvpn.lib.model.MultihopRelayListType
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.model.RelayItemSelection
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItemState
import net.mullvad.mullvadvpn.relaylist.filterOnSearchTerm

const val RECENTS_MAX_VISIBLE: Int = 3

// Creates a relay list to be displayed by RelayListContent
internal fun relayListItems(
    relayListType: RelayListType,
    relayCountries: List<RelayItem.Location.Country>,
    customLists: List<RelayItem.CustomList>,
    recents: List<Hop.Single<RelayItem>>?,
    selectedItem: RelayItemSelection,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    expandedItems: Set<String>,
): List<RelayListItem> {
    return createRelayListItems(
        relayListType = relayListType,
        selectedItem = selectedItem,
        selectedByThisEntryExitList = selectedByThisEntryExitList,
        selectedByOtherEntryExitList = selectedByOtherEntryExitList,
        customLists = customLists,
        recents = recents,
        countries = relayCountries,
    ) {
        it in expandedItems
    }
}

internal fun relayListItemsSearching(
    searchTerm: String = "",
    relayListType: RelayListType,
    relayCountries: List<RelayItem.Location.Country>,
    customLists: List<RelayItem.CustomList>,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    expandedItems: Set<String>,
): List<RelayListItem> {
    val filteredCustomLists = customLists.filterOnSearchTerm(searchTerm)

    return createRelayListItemsSearching(
            relayListType = relayListType,
            selectedByThisEntryExitList = selectedByThisEntryExitList,
            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
            customLists = filteredCustomLists,
            countries = relayCountries,
        ) {
            it in expandedItems
        }
        .ifEmpty { listOf(RelayListItem.LocationsEmptyText(searchTerm)) }
}

internal fun emptyLocationsRelayListItems(
    relayListType: RelayListType,
    customLists: List<RelayItem.CustomList>,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    expandedItems: Set<String>,
) =
    createCustomListSection(
        relayListType,
        selectedByThisEntryExitList,
        selectedByOtherEntryExitList,
        customLists,
    ) {
        it in expandedItems
    } + RelayListItem.LocationHeader + RelayListItem.EmptyRelayList

private fun createRelayListItems(
    relayListType: RelayListType,
    selectedItem: RelayItemSelection,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    recents: List<Hop.Single<RelayItem>>?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (recents != null) {
        addAll(createRecentsSection(recents, selectedItem, relayListType))
    }
    addAll(
        createCustomListSection(
            relayListType,
            selectedByThisEntryExitList,
            selectedByOtherEntryExitList,
            customLists,
            isExpanded,
        )
    )
    addAll(
        createLocationSection(
            selectedByThisEntryExitList,
            relayListType,
            selectedByOtherEntryExitList,
            countries,
            isExpanded,
        )
    )
}

private fun createRecentsSection(
    recents: List<Hop.Single<RelayItem>>,
    itemSelection: RelayItemSelection,
    relayListType: RelayListType,
): List<RelayListItem> = buildList {
    add(RelayListItem.RecentsListHeader)

    val shown =
        recents
            .map { recent ->
                val isSelected = recent.matches(itemSelection, relayListType)
                RelayListItem.RecentListItem(hop = recent, isSelected = isSelected)
            }
            // Convert to a set to remove possible duplicates. We can get duplicate entries if
            // multihop is enabled because multiple multihop recents
            // can have the same entry or exit.
            .toSet()
            .take(RECENTS_MAX_VISIBLE)

    addAll(shown)
    if (shown.isEmpty()) {
        add(RelayListItem.RecentsListFooter)
    } else {
        add(RelayListItem.SectionDivider())
    }
}

private fun Hop.Single<RelayItem>.matches(
    itemSelection: RelayItemSelection,
    relayListType: RelayListType,
): Boolean {
    return when (itemSelection) {
        is RelayItemSelection.Single -> {
            relay.id == itemSelection.exitLocation.getOrNull()
        }

        is RelayItemSelection.Multiple if relayListType is RelayListType.Multihop -> {
            if (relayListType.multihopRelayListType == MultihopRelayListType.ENTRY) {
                relay.id == itemSelection.entryLocation.getOrNull()
            } else {
                relay.id == itemSelection.exitLocation.getOrNull()
            }
        }

        else -> false
    }
}

private fun createRelayListItemsSearching(
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> =
    createCustomListSectionSearching(
        relayListType,
        selectedByThisEntryExitList,
        selectedByOtherEntryExitList,
        customLists,
        isExpanded,
    ) +
        createLocationSectionSearching(
            selectedByThisEntryExitList,
            relayListType,
            selectedByOtherEntryExitList,
            countries,
            isExpanded,
        )

private fun createCustomListSection(
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
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
    add(RelayListItem.CustomListFooter(customListItems.isNotEmpty()))
}

private fun createCustomListSectionSearching(
    relayListType: RelayListType,
    selectedByThisEntryExitList: RelayItemId?,
    selectedByOtherEntryExitList: RelayItemId?,
    customLists: List<RelayItem.CustomList>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (customLists.isNotEmpty()) {
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
        // It is possible for an custom list to be expanded without children if the children were
        // removed after the item was expanded. In those cases we should treat the item as
        // collapsed.
        val expanded = isExpanded(customList.id.expandKey()) && customList.hasChildren
        buildList {
            add(
                RelayListItem.CustomListItem(
                    hop = Hop.Single(customList),
                    isSelected = selectedByThisEntryExitList == customList.id,
                    state =
                        customList.createState(
                            relayListType = relayListType,
                            selectedByOtherId = selectedByOtherEntryExitList,
                        ),
                    expanded = expanded,
                    itemPosition =
                        if (expanded) {
                            ItemPosition.Top
                        } else {
                            ItemPosition.Single
                        },
                )
            )

            if (expanded) {
                addAll(
                    customList.locations.flatMapIndexed { index, item ->
                        createCustomListEntry(
                            parent = customList,
                            item = item,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = 1,
                            isExpanded = isExpanded,
                            isLast = index == customList.locations.lastIndex,
                        )
                    }
                )
            }
        }
    }

private fun createLocationSection(
    selectedByThisEntryExitList: RelayItemId?,
    relayListType: RelayListType,
    selectedByOtherEntryExitList: RelayItemId?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    add(RelayListItem.LocationHeader)
    addAll(
        countries.flatMapIndexed { index, country ->
            createGeoLocationEntry(
                item = country,
                selectedByThisEntryExitList = selectedByThisEntryExitList,
                relayListType = relayListType,
                selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                isExpanded = isExpanded,
                isLast = true,
            )
        }
    )
}

private fun createLocationSectionSearching(
    selectedByThisEntryExitList: RelayItemId?,
    relayListType: RelayListType,
    selectedByOtherEntryExitList: RelayItemId?,
    countries: List<RelayItem.Location.Country>,
    isExpanded: (String) -> Boolean,
): List<RelayListItem> = buildList {
    if (countries.isNotEmpty()) {
        add(RelayListItem.LocationHeader)
        addAll(
            countries.flatMap { country ->
                createGeoLocationEntry(
                    item = country,
                    selectedByThisEntryExitList = selectedByThisEntryExitList,
                    relayListType = relayListType,
                    selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                    isExpanded = isExpanded,
                    isLast = true,
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
    isLast: Boolean,
): List<RelayListItem.CustomListEntryItem> = buildList {
    val expanded = isExpanded(item.id.expandKey(parent.id))
    add(
        RelayListItem.CustomListEntryItem(
            parentId = parent.id,
            parentName = parent.customList.name,
            hop = Hop.Single(item),
            state =
                item.createState(
                    relayListType = relayListType,
                    selectedByOtherId = selectedByOtherEntryExitList,
                ),
            expanded = expanded,
            depth = depth,
            itemPosition =
                if (!expanded && isLast) {
                    ItemPosition.Bottom
                } else {
                    ItemPosition.Middle
                },
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMapIndexed { index, relay ->
                        createCustomListEntry(
                            parent = parent,
                            item = relay,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                            isLast = isLast && index == item.relays.lastIndex,
                        )
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMapIndexed { index, city ->
                        createCustomListEntry(
                            parent = parent,
                            item = city,
                            relayListType = relayListType,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                            isLast = isLast && index == item.cities.lastIndex,
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
    isLast: Boolean,
): List<RelayListItem.GeoLocationItem> = buildList {
    val expanded = isExpanded(item.id.expandKey())

    add(
        RelayListItem.GeoLocationItem(
            hop = Hop.Single(item),
            isSelected = selectedByThisEntryExitList == item.id,
            state =
                item.createState(
                    relayListType = relayListType,
                    selectedByOtherId = selectedByOtherEntryExitList,
                ),
            depth = depth,
            expanded = expanded,
            itemPosition =
                when (item) {
                    is RelayItem.Location.Country -> {
                        if (expanded) {
                            ItemPosition.Top
                        } else {
                            ItemPosition.Single
                        }
                    }

                    else -> {
                        if (isLast && !expanded) {
                            ItemPosition.Bottom
                        } else {
                            ItemPosition.Middle
                        }
                    }
                },
        )
    )

    if (expanded) {
        when (item) {
            is RelayItem.Location.City ->
                addAll(
                    item.relays.flatMapIndexed { index, relay ->
                        createGeoLocationEntry(
                            item = relay,
                            relayListType = relayListType,
                            selectedByThisEntryExitList = selectedByThisEntryExitList,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                            isLast = isLast && index == item.relays.lastIndex,
                        )
                    }
                )
            is RelayItem.Location.Country ->
                addAll(
                    item.cities.flatMapIndexed { index, city ->
                        createGeoLocationEntry(
                            item = city,
                            relayListType = relayListType,
                            selectedByThisEntryExitList = selectedByThisEntryExitList,
                            selectedByOtherEntryExitList = selectedByOtherEntryExitList,
                            depth = depth + 1,
                            isExpanded = isExpanded,
                            isLast = isLast && index == item.cities.lastIndex,
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
            when ((relayListType as? RelayListType.Multihop)?.multihopRelayListType) {
                MultihopRelayListType.ENTRY -> entryLocation.getOrNull()
                MultihopRelayListType.EXIT -> exitLocation.getOrNull()
                else -> null
            }
        is RelayItemSelection.Single -> exitLocation.getOrNull()
    }

internal fun RelayItemSelection.selectedByOtherEntryExitList(
    relayListType: RelayListType,
    customLists: List<RelayItem.CustomList>,
) =
    when (this) {
        is RelayItemSelection.Multiple -> {
            val location =
                when ((relayListType as? RelayListType.Multihop)?.multihopRelayListType) {
                    MultihopRelayListType.ENTRY -> exitLocation
                    MultihopRelayListType.EXIT -> entryLocation
                    else -> null
                }?.getOrNull()
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
        when ((relayListType as? RelayListType.Multihop)?.multihopRelayListType) {
            MultihopRelayListType.ENTRY -> RelayListItemState.USED_AS_EXIT
            MultihopRelayListType.EXIT -> RelayListItemState.USED_AS_ENTRY
            else -> null
        }
    } else {
        null
    }
}
