package net.mullvad.mullvadvpn.compose.data

import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.ItemPosition
import net.mullvad.mullvadvpn.lib.ui.component.relaylist.RelayListItem
import net.mullvad.mullvadvpn.relaylist.descendants

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
            add(RelayListItem.RecentListItem(it, isSelected = it.id == selectedItem))
        }
    }
    if (customListItem.isNotEmpty()) {
        add(RelayListItem.CustomListHeader)
        customListItem.forEach {
            add(RelayListItem.CustomListItem(it, isSelected = it.id == selectedItem))
        }
        add(RelayListItem.CustomListFooter(hasCustomList = true))
    }
    if (locationItems.isNotEmpty()) {
        add(RelayListItem.LocationHeader)
        locationItems.forEach { country ->
            val descendantIsSelected = country.descendants().any { it.id == selectedItem }
            add(
                RelayListItem.GeoLocationItem(
                    item = country,
                    isSelected = country == selectedItem,
                    expanded = descendantIsSelected,
                    itemPosition =
                        if (descendantIsSelected) {
                            ItemPosition.Top
                        } else {
                            ItemPosition.Single
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
                            itemPosition =
                                if (country.cities.last() == city && !childIsSelected) {
                                    ItemPosition.Bottom
                                } else {
                                    ItemPosition.Middle
                                },
                        )
                    )
                    if (childIsSelected) {
                        city.relays.forEach { relay ->
                            add(
                                RelayListItem.GeoLocationItem(
                                    item = relay,
                                    isSelected = relay.id == selectedItem,
                                    itemPosition =
                                        if (city.relays.last() == relay) {
                                            ItemPosition.Bottom
                                        } else {
                                            ItemPosition.Middle
                                        },
                                )
                            )
                        }
                    }
                }
            }
        }
    }
}
