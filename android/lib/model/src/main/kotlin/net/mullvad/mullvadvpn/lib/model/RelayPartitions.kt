package net.mullvad.mullvadvpn.lib.model

data class RelayPartitions(val matches: List<String>, val discards: List<DiscardedRelay>)
