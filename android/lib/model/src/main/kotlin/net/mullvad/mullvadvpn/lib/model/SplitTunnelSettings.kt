package net.mullvad.mullvadvpn.lib.model

enum class SplitTunnelMode {
    /** Listed apps bypass the VPN tunnel (default). */
    EXCLUDE,
    /** Only listed apps use the VPN tunnel; all other traffic bypasses it. */
    INCLUDE,
}

data class SplitTunnelSettings(
    val enabled: Boolean,
    val excludedApps: Set<AppId>,
    val mode: SplitTunnelMode = SplitTunnelMode.EXCLUDE,
)
