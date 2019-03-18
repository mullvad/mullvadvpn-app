package net.mullvad.mullvadvpn.relaylist

data class Relay(override val name: String) : RelayItem {
    override val code = name
    override val type = RelayItemType.Relay
    override val hasChildren = false

    override val visibleChildCount = 0

    override var expanded
        get() = false
        set(value) {}
}
