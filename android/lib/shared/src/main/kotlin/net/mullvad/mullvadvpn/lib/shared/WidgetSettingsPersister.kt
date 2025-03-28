package net.mullvad.mullvadvpn.lib.shared

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.update

class WidgetSettingsPersister {
    private val _widgetSettingsState = MutableStateFlow(WidgetSettingsState())
    val widgetSettingsState: StateFlow<WidgetSettingsState> = _widgetSettingsState

    fun setShowLan(show: Boolean) {
        _widgetSettingsState.update { it.copy(showLan = show) }
    }

    fun setShowCustomDns(show: Boolean) {
        _widgetSettingsState.update { it.copy(showCustomDns = show) }
    }

    fun setShowDaita(show: Boolean) {
        _widgetSettingsState.update { it.copy(showDaita = show) }
    }

    fun setShowSplitTunneling(show: Boolean) {
        _widgetSettingsState.update { it.copy(showSplitTunneling = show) }
    }

    companion object {
        val SINGLETON = WidgetSettingsPersister()
    }
}

data class WidgetSettingsState(
    val showLan: Boolean = true,
    val showCustomDns: Boolean = true,
    val showDaita: Boolean = true,
    val showSplitTunneling: Boolean = true,
)
