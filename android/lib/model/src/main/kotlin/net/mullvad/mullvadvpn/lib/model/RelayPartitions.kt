package net.mullvad.mullvadvpn.lib.model

data class RelayPartitions(
    val matches: List<GeoLocationId.Hostname>,
    val discards: List<DiscardedRelay>,
)
