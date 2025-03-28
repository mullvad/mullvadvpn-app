package net.mullvad.mullvadvpn.lib.shared

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.lib.model.Settings

class WidgetRepository(
    managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO,
) {
    val settingsUpdates: StateFlow<Settings?> =
        managementService.settings.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.Companion.WhileSubscribed(),
            null,
        )
}
