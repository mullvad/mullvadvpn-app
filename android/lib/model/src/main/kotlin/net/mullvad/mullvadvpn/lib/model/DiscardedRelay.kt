package net.mullvad.mullvadvpn.lib.model

data class DiscardedRelay(val hostname: String, val why: IncompatibleConstraints)
