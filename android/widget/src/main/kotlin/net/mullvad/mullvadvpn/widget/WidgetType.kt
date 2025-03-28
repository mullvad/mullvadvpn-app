package net.mullvad.mullvadvpn.widget

enum class WidgetType(val className: String) {
    SETTINGS(".widget.SettingsWidgetReceiver"),
    DNS_CONTENT_BLOCKERS(".widget.DnsContentBlockersWidgetReceiver");

    companion object {
        fun fromClass(className: String): WidgetType {
            return entries.first { it.className == className }
        }
    }
}
