package net.mullvad.mullvadvpn.lib.model

@JvmInline
value class ProviderId(val value: String) : Comparable<ProviderId> {
    override fun compareTo(other: ProviderId): Int =
        value.uppercase().compareTo(other.value.uppercase())
}
