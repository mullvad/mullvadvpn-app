package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

interface RelayItem {
    val type: RelayItemType
    val name: String
    val code: String
    val location: GeographicLocationConstraint
    val active: Boolean
    val hasChildren: Boolean
    val visibleChildCount: Int

    val visibleItemCount: Int
        get() = visibleChildCount + 1

    val locationName: String
        get() = name

    var expanded: Boolean
}
