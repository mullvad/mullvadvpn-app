package net.mullvad.mullvadvpn.widget

import net.mullvad.mullvadvpn.lib.shared.WidgetRepository
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsPersister

class WidgetProvider(private val widgetRepository: WidgetRepository) {
    fun settings() = widgetRepository.settingsUpdates

    fun widgetSettings() = WidgetSettingsPersister.SINGLETON.widgetSettingsState
}
