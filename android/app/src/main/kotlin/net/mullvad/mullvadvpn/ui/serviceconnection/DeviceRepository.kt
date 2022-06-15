package net.mullvad.mullvadvpn.ui.serviceconnection

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onEach
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.DeviceState

class DeviceRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    private val deviceListTimeoutMillis: Long = 5000L,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val cachedDeviceList = MutableStateFlow<List<Device>>(emptyList())

    val deviceState = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.deviceDataSource.deviceStateUpdates
                    .onStart {
                        state.container.deviceDataSource.getDevice()
                    }
            } else {
                flowOf(DeviceState.Unknown)
            }
        }
        .stateIn(
            CoroutineScope(dispatcher),
            SharingStarted.WhileSubscribed(),
            DeviceState.Initial
        )

    private val deviceListEvents = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.deviceDataSource.deviceListUpdates
            } else {
                emptyFlow()
            }
        }

    val deviceList = deviceListEvents
        .map { (it as? DeviceListEvent.Available)?.devices ?: emptyList() }
        .onStart {
            if (cachedDeviceList.value.isNotEmpty()) {
                emit(cachedDeviceList.value)
            }
        }
        .stateIn(CoroutineScope(Dispatchers.IO), SharingStarted.WhileSubscribed(), emptyList())

    fun refreshDeviceState() {
        container()?.deviceDataSource?.refreshDevice()
    }

    private fun container(): ServiceConnectionContainer? {
        return serviceConnectionManager.connectionState.value.readyContainer()
    }

    fun removeDevice(accountToken: String, deviceId: String) {
        cachedDeviceList.value = emptyList()
        container()?.deviceDataSource?.removeDevice(accountToken, deviceId)
    }

    fun refreshDeviceList(accountToken: String) {
        container()?.deviceDataSource?.refreshDeviceList(accountToken)
    }

    suspend fun getDeviceList(accountToken: String): DeviceListEvent {
        return withTimeoutOrNull(deviceListTimeoutMillis) {
            deviceListEvents
                .onStart {
                    refreshDeviceList(accountToken)
                }
                .onEach {
                    cachedDeviceList.value =
                        (it as? DeviceListEvent.Available)?.devices ?: emptyList()
                }
                .firstOrNull() ?: DeviceListEvent.Error
        } ?: DeviceListEvent.Error
    }
}
