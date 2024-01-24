package net.mullvad.mullvadvpn.relaylist

data class RelayList(
    val customLists: List<CustomRelayItemList>,
    val country: List<RelayCountry>,
    val selectedItem: SelectedLocation?,
)
