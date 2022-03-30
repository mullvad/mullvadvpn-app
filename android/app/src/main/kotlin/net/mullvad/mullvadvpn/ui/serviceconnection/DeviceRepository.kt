package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.SharingStarted.Companion.Lazily
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.model.DeviceState

class DeviceRepository(
    private val dataSource: ServiceConnectionDeviceDataSource,
    externalScope: CoroutineScope
) {
    val deviceState = dataSource.deviceStateUpdates
        .stateIn(
            externalScope,
            Lazily,
            DeviceState.InitialState
        )

    fun refreshDeviceState() = dataSource.refreshDevice()
}
