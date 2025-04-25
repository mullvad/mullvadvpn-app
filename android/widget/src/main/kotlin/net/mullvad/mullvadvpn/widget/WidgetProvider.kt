package net.mullvad.mullvadvpn.widget

import net.mullvad.mullvadvpn.lib.shared.WidgetRepository

class WidgetProvider(private val widgetRepository: WidgetRepository) {
    fun settings() = widgetRepository.settingsUpdates
}
