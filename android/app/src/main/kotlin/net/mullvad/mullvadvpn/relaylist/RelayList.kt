package net.mullvad.mullvadvpn.relaylist

data class RelayList(
    val customLists: List<RelayItem.CustomList>,
    val allCountries: List<RelayItem.Country>,
    val filteredCountries: List<RelayItem.Country>,
    val selectedItem: RelayItem?,
)
