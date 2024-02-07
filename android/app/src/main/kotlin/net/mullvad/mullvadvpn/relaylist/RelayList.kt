package net.mullvad.mullvadvpn.relaylist

data class RelayList(
    val customLists: List<RelayItem.CustomList>,
    val country: List<RelayItem.Country>,
    val selectedItem: RelayItem?,
)
