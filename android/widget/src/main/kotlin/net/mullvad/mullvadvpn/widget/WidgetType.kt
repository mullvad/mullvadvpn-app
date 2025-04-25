package net.mullvad.mullvadvpn.widget

enum class WidgetType(val className: String) {
    SETTINGS(".widget.SettingsWidgetReceiver");

    companion object {
        fun fromClass(className: String): WidgetType {
            return entries.first { it.className == className }
        }
    }
}
