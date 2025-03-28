package net.mullvad.mullvadvpn.widget

import android.content.Context
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.shared.WidgetRepository
import net.mullvad.mullvadvpn.lib.shared.WidgetSettingsPersister

class MullvadWidgetUpdater(
    private val context: Context,
    private val widgetRepository: WidgetRepository,
    private val scope: CoroutineScope,
) {
    private var job: Job? = null

    fun start() {
        if (job != null) {
            error("MullvadWidgetUpdater already started")
        }

        job = scope.launch { launchListenToSettings() }
        job = scope.launch { launchListenToWidgetSettings() }
    }

    fun stop() {
        job?.cancel(message = "MullvadWidgetUpdater stopped")
            ?: error("MullvadWidgetUpdater already stopped")
        job = null
    }

    private suspend fun launchListenToSettings() {
        widgetRepository.settingsUpdates
            .onStart { null }
            .collect { MullvadAppWidget.updateAllWidgets(context) }
    }

    private suspend fun launchListenToWidgetSettings() {
        WidgetSettingsPersister.SINGLETON.widgetSettingsState.collect {
            MullvadAppWidget.updateAllWidgets(context)
        }
    }
}
