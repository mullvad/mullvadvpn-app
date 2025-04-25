package net.mullvad.mullvadvpn.lib.shared

data class WidgetSettingsState(
    val showLan: Boolean,
    val showCustomDns: Boolean,
    val showDaita: Boolean,
    val showSplitTunneling: Boolean,
    val showMultihop: Boolean,
    val showInTunnelIpv6: Boolean,
    val showQuantumResistant: Boolean,
) {
    fun toPrefs(): Set<String> =
        setOf(
            if (showLan) SHOW_LAN else "",
            if (showCustomDns) SHOW_CUSTOM_DNS else "",
            if (showDaita) SHOW_DAITA else "",
            if (showSplitTunneling) SHOW_SPLIT_TUNNELING else "",
            if (showMultihop) SHOW_MULTIHOP else "",
            if (showInTunnelIpv6) SHOW_IN_TUNNEL_IPV6 else "",
            if (showQuantumResistant) SHOW_QUANTUM_RESISTANT else "",
        )

    fun anyShowing() =
        showLan ||
            showCustomDns ||
            showDaita ||
            showSplitTunneling ||
            showMultihop ||
            showInTunnelIpv6

    companion object {
        const val PREF_KEY = "widget_settings"

        const val SHOW_LAN = "showLan"
        const val SHOW_CUSTOM_DNS = "showCustomDns"
        const val SHOW_DAITA = "showDaita"
        const val SHOW_SPLIT_TUNNELING = "showSplitTunneling"
        const val SHOW_MULTIHOP = "showMultihop"
        const val SHOW_IN_TUNNEL_IPV6 = "showInTunnelIpv6"
        const val SHOW_QUANTUM_RESISTANT = "showQuantumResistant"

        fun fromPrefs(prefs: Set<String>) =
            WidgetSettingsState(
                showLan = prefs.contains(SHOW_LAN),
                showCustomDns = prefs.contains(SHOW_CUSTOM_DNS),
                showDaita = prefs.contains(SHOW_DAITA),
                showSplitTunneling = prefs.contains(SHOW_SPLIT_TUNNELING),
                showMultihop = prefs.contains(SHOW_MULTIHOP),
                showInTunnelIpv6 = prefs.contains(SHOW_IN_TUNNEL_IPV6),
                showQuantumResistant = prefs.contains(SHOW_QUANTUM_RESISTANT),
            )
    }
}
