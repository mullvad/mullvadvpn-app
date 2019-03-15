package net.mullvad.mullvadvpn.relaylist

class RelayCity(val city: String, val relays: List<Relay>, var expanded: Boolean) {
    fun getItem(position: Int): GetItemResult {
        if (position == 0) {
            return GetItemResult.Item(RelayItem(RelayItemType.City, city))
        }

        if (!expanded) {
            return GetItemResult.Count(1)
        }

        val offset = position - 1
        val relayCount = relays.size

        if (offset >= relayCount) {
            return GetItemResult.Count(1 + relayCount)
        } else {
            return GetItemResult.Item(RelayItem(RelayItemType.Relay, relays[offset].hostname))
        }
    }

    fun getItemCount(): Int {
        if (expanded) {
            return 1 + relays.size
        } else {
            return 1
        }
    }
}
