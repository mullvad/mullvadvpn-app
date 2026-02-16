package net.mullvad.mullvadvpn.feature.location.impl.data

import net.mullvad.mullvadvpn.lib.common.util.relaylist.descendants
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Suppress("CyclomaticComplexMethod")
fun createSimpleRelayListItemList(
    recentItems: List<RelayItem.Location> = emptyList(),
    customListItem: List<RelayItem.CustomList> = emptyList(),
    locationItems: List<RelayItem.Location.Country> = emptyList(),
    selectedItem: RelayItemId? = null,
): List<RelayListItem> = buildList {
    if (recentItems.isNotEmpty()) {
        add(RelayListItem.RecentsListHeader)
        recentItems.forEach {
            add(RelayListItem.RecentListItem(item = it, isSelected = it.id == selectedItem))
        }
    }
    add(RelayListItem.CustomListHeader)
    if (customListItem.isNotEmpty()) {
        customListItem.forEach {
            add(RelayListItem.CustomListItem(it, isSelected = it.id == selectedItem))
        }
    }
    add(RelayListItem.CustomListFooter(hasCustomList = customListItem.isNotEmpty()))
    add(RelayListItem.LocationHeader)
    if (locationItems.isNotEmpty()) {
        locationItems.forEach { country ->
            val descendantIsSelected = country.descendants().any { it.id == selectedItem }
            add(
                RelayListItem.GeoLocationItem(
                    item = country,
                    isSelected = country == selectedItem,
                    expanded = descendantIsSelected,
                    hierarchy = Hierarchy.Parent,
                    itemPosition =
                        if (descendantIsSelected) {
                            Position.Top
                        } else {
                            Position.Single
                        },
                )
            )
            if (descendantIsSelected) {
                country.cities.forEach { city ->
                    val childIsSelected = city.relays.any { it.id == selectedItem }
                    add(
                        RelayListItem.GeoLocationItem(
                            item = city,
                            isSelected = city.id == selectedItem,
                            expanded = childIsSelected,
                            hierarchy = Hierarchy.Parent,
                            itemPosition =
                                if (country.cities.last() == city && !childIsSelected) {
                                    Position.Bottom
                                } else {
                                    Position.Middle
                                },
                        )
                    )
                    if (childIsSelected) {
                        city.relays.forEach { relay ->
                            add(
                                RelayListItem.GeoLocationItem(
                                    item = relay,
                                    isSelected = relay.id == selectedItem,
                                    hierarchy = Hierarchy.Parent,
                                    itemPosition =
                                        if (city.relays.last() == relay) {
                                            Position.Bottom
                                        } else {
                                            Position.Middle
                                        },
                                )
                            )
                        }
                    }
                }
            }
        }
    } else {
        add(RelayListItem.EmptyRelayList)
    }
}
