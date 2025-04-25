package net.mullvad.mullvadvpn.app.widget

enum class WidgetType(val className: String) {
    SETTINGS(".widget.SettingsWidgetReceiver");

    companion object {
        fun fromClass(className: String): WidgetType {
            return entries.first { it.className == className }
        }
    }
}
