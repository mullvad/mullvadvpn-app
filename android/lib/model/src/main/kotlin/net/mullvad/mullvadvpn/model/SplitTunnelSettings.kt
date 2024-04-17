package net.mullvad.mullvadvpn.model

data class SplitTunnelSettings(val enabled: Boolean, val excludedApps: Set<AppId>)
