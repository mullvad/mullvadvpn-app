package net.mullvad.mullvadvpn.lib.model

data class DiscardedRelay(val hostname: GeoLocationId.Hostname, val why: IncompatibleConstraints)
