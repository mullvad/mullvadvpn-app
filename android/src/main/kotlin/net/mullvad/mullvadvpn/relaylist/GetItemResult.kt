package net.mullvad.mullvadvpn.relaylist

sealed class GetItemResult {
    data class Item(val item: RelayItem) : GetItemResult()
    data class Count(val count: Int) : GetItemResult()
}
