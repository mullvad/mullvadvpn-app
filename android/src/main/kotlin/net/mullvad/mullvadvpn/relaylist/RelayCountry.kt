package net.mullvad.mullvadvpn.relaylist

class RelayCountry(
    override val name: String,
    override val code: String,
    override var expanded: Boolean,
    val cities: List<RelayCity>
) : RelayItem {
    override val type = RelayItemType.Country
    override val hasChildren
        get() = getRelayCount() > 1

    override val visibleChildCount: Int
        get() {
            if (expanded) {
                return cities.map { city -> city.visibleItemCount }.sum()
            } else {
                return 0
            }
        }

    fun getItem(position: Int): GetItemResult {
        if (position == 0) {
            return GetItemResult.Item(this)
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

    fun getRelayCount(): Int = cities.map { city -> city.getRelayCount() }.sum()
}
