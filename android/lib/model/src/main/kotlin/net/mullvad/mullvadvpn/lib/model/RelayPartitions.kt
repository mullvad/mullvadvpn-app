package net.mullvad.mullvadvpn.lib.model

typealias PartitionHostname = String

typealias NeedsOtherEntry = Boolean

data class RelayPartitions(
    val matches: Map<PartitionHostname, NeedsOtherEntry>,
    val discards: List<DiscardedRelay>,
)
