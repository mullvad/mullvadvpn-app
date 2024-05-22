package net.mullvad.mullvadvpn.lib.model

data class SplitTunnelSettings(val enabled: Boolean, val excludedApps: Set<AppId>)
