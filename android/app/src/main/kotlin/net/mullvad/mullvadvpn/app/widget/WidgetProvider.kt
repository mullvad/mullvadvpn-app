package net.mullvad.mullvadvpn.widget

import net.mullvad.mullvadvpn.lib.repository.WidgetRepository

class WidgetProvider(private val widgetRepository: WidgetRepository) {
    fun settings() = widgetRepository.settingsUpdates
}
