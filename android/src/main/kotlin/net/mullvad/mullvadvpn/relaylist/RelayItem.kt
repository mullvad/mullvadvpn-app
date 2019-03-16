package net.mullvad.mullvadvpn.relaylist

interface RelayItem {
    val type: RelayItemType
    val name: String
    val hasChildren: Boolean
    val visibleChildCount: Int

    val visibleItemCount: Int
        get() = visibleChildCount + 1

    var expanded: Boolean
}
