package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

interface RelayItem {
    val type: RelayItemType
    val name: String
    val code: String
    val location: LocationConstraint
    val hasChildren: Boolean
    val visibleChildCount: Int

    val visibleItemCount: Int
        get() = visibleChildCount + 1

    var expanded: Boolean
}
