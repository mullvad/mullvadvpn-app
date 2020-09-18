package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.LocationConstraint

class RelayCity(
    val country: RelayCountry,
    override val name: String,
    override val code: String,
    override var expanded: Boolean,
    val relays: List<Relay>
) : RelayItem {
    override val type = RelayItemType.City
    override val location = LocationConstraint.City(country.code, code)

    override val active
        get() = relays.any { relay -> relay.active }

    override val hasChildren
        get() = relays.size > 0

    override val visibleChildCount: Int
        get() {
            if (expanded) {
                return relays.size
            } else {
                return 0
            }
        }

    fun getItem(position: Int): GetItemResult {
        if (position == 0) {
            return GetItemResult.Item(this)
        }

        if (!expanded) {
            return GetItemResult.Count(1)
        }

        val offset = position - 1
        val relayCount = relays.size

        if (offset >= relayCount) {
            return GetItemResult.Count(1 + relayCount)
        } else {
            return GetItemResult.Item(relays[offset])
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
