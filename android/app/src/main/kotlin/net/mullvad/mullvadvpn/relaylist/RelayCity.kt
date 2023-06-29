package net.mullvad.mullvadvpn.relaylist

import net.mullvad.mullvadvpn.model.GeographicLocationConstraint

class RelayCity(
    val country: RelayCountry,
    override val name: String,
    override val code: String,
    override var expanded: Boolean,
    val relays: List<Relay>
) : RelayItem {
    override val type = RelayItemType.City
    override val location = GeographicLocationConstraint.City(country.code, code)

    override val active
        get() = relays.any { relay -> relay.active }

    override val hasChildren
        get() = relays.isNotEmpty()

    override val visibleChildCount: Int
        get() {
            return if (expanded) {
                relays.size
            } else {
                0
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

        return if (offset >= relayCount) {
            GetItemResult.Count(1 + relayCount)
        } else {
            GetItemResult.Item(relays[offset])
        }
    }

    fun getItemCount(): Int {
        return if (expanded) {
            1 + relays.size
        } else {
            1
        }
    }
}
