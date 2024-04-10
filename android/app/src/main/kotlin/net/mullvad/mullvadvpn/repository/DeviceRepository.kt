package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.lib.daemon.grpc.ManagementService
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.model.ListDevicesError
import net.mullvad.mullvadvpn.model.RemoveDeviceEvent

class DeviceRepository(
    private val managementService: ManagementService,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    val deviceState: StateFlow<DeviceState> =
        managementService.deviceState.stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.Eagerly,
            DeviceState.Initial
        )

    private val _devices = Channel<List<Device>?>()
    val devices: StateFlow<List<Device>?> =
        _devices.receiveAsFlow().stateIn(CoroutineScope(dispatcher), SharingStarted.Eagerly, null)

    val deviceRemovalEvent: SharedFlow<RemoveDeviceEvent> =
        managementService.removeDeviceEvent
            .onEach { _devices.send(it.newDevices) }
            .shareIn(CoroutineScope(dispatcher), SharingStarted.WhileSubscribed())

    suspend fun removeDevice(accountToken: String, deviceId: String) =
        managementService.removeDevice(accountToken, deviceId)

    suspend fun refreshDeviceList(accountToken: String): ListDevicesError? {
        return managementService
            .listDevices(accountToken)
            .fold(
                { it },
                {
                    _devices.send(it)
                    null
                }
            )
    }
}
