package net.mullvad.mullvadvpn.repository

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.firstOrNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.shareIn
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.model.DeviceList
import net.mullvad.mullvadvpn.model.DeviceListEvent
import net.mullvad.mullvadvpn.model.DeviceState
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionState
import net.mullvad.mullvadvpn.ui.serviceconnection.deviceDataSource

class DeviceRepository(
    private val serviceConnectionManager: ServiceConnectionManager,
    dispatcher: CoroutineDispatcher = Dispatchers.IO
) {
    private val cachedDeviceList = MutableStateFlow<DeviceList>(DeviceList.Unavailable)

    val deviceState = serviceConnectionManager.connectionState
        .flatMapLatest { state ->
            if (state is ServiceConnectionState.ConnectedReady) {
                state.container.deviceDataSource.deviceStateUpdates
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
        .map {
            if (it is DeviceListEvent.Available) {
                DeviceList.Available(it.devices)
            } else {
                DeviceList.Error
            }
        }
        .onStart {
            if (cachedDeviceList.value is DeviceList.Available) {
                emit(cachedDeviceList.value)
            }
        }
        .shareIn(
            CoroutineScope(Dispatchers.IO),
            SharingStarted.WhileSubscribed()
        )

    val deviceRemovalEvent: SharedFlow<Event.DeviceRemovalEvent> =
        serviceConnectionManager.connectionState
            .flatMapLatest { state ->
                if (state is ServiceConnectionState.ConnectedReady) {
                    state.container.deviceDataSource.deviceRemovalResult
                } else {
                    emptyFlow()
                }
            }
            .shareIn(
                CoroutineScope(dispatcher),
                SharingStarted.WhileSubscribed()
            )

    fun refreshDeviceState() {
        serviceConnectionManager.deviceDataSource()?.refreshDevice()
    }

    fun removeDevice(accountToken: String, deviceId: String) {
        serviceConnectionManager.deviceDataSource()?.removeDevice(accountToken, deviceId)
    }

    fun refreshDeviceList(accountToken: String) {
        serviceConnectionManager.deviceDataSource()?.refreshDeviceList(accountToken)
    }

    fun clearCache() {
        cachedDeviceList.value = DeviceList.Unavailable
    }

    private fun updateCache(event: DeviceListEvent, accountToken: String) {
        cachedDeviceList.value =
            if (event is DeviceListEvent.Available && event.accountToken == accountToken) {
                DeviceList.Available(event.devices)
            } else if (event is DeviceListEvent.Error) {
                DeviceList.Error
            } else {
                DeviceList.Unavailable
            }
    }

    suspend fun refreshAndAwaitDeviceListWithTimeout(
        accountToken: String,
        shouldClearCache: Boolean,
        shouldOverrideCache: Boolean,
        timeoutMillis: Long,
    ): DeviceListEvent {
        if (shouldClearCache) {
            clearCache()
        }

        val result = withTimeoutOrNull(timeoutMillis) {
            deviceListEvents
                .onStart {
                    refreshDeviceList(accountToken)
                }
                .firstOrNull() ?: DeviceListEvent.Error
        } ?: DeviceListEvent.Error

        if (shouldOverrideCache) {
            updateCache(result, accountToken)
        }

        return result
    }
}
