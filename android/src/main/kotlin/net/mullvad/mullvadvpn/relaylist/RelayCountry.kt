package net.mullvad.mullvadvpn.relaylist

class RelayCountry(val country: String, val cities: List<RelayCity>, var expanded: Boolean) {
    fun getItem(position: Int): GetItemResult {
        if (position == 0) {
            return GetItemResult.Item(RelayItem(RelayItemType.Country, country))
        }

        var itemCount = 1
        var remaining = position - 1

        if (expanded) {
            for (city in cities) {
                val itemOrCount = city.getItem(remaining)

                when (itemOrCount) {
                    is GetItemResult.Item -> return itemOrCount
                    is GetItemResult.Count -> {
                        remaining -= itemOrCount.count
                        itemCount += itemOrCount.count
                    }
                }
            }
        }

        return GetItemResult.Count(itemCount)
    }

    fun getItemCount(): Int {
        if (expanded) {
            return 1 + cities.map { city -> city.getItemCount() }.sum()
        } else {
            return 1
        }
    }
}
