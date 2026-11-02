package net.mullvad.mullvadvpn.lib.model

import java.time.Instant

data class TunnelStats(val rx: Long = 0, val tx: Long = 0, val lastHandshake: Instant? = null)
