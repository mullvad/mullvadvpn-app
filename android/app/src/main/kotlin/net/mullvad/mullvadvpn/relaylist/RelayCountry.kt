package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

data class RelayCountry(
    override val name: String,
    override val code: String,
    override var expanded: Boolean,
    val cities: List<RelayCity>
) : RelayItem {
    override val type = RelayItemType.Country
    override val location = GeographicLocationConstraint.Country(code)

    override val active
        get() = cities.any { city -> city.active }

    override val hasChildren
        get() = cities.isNotEmpty()

    override val visibleChildCount: Int
        get() {
            return if (expanded) {
                cities.sumOf { city -> city.visibleItemCount }
            } else {
                0
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

                when (val itemOrCount = city.getItem(remaining)) {
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
}
