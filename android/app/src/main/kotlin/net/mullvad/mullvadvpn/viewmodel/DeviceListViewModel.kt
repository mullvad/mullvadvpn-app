package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.Either
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import net.mullvad.mullvadvpn.compose.state.DeviceListItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.common.util.parseAsDateTime
import net.mullvad.mullvadvpn.model.ListDevicesError
import net.mullvad.mullvadvpn.repository.DeviceRepository

typealias DeviceId = String

class DeviceListViewModel(
    accountToken: String,
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default
) : ViewModel() {
    private val _loadingDevices = MutableStateFlow<List<DeviceId>>(emptyList())
    private val _listDevicesError = MutableStateFlow<ListDevicesError?>(null)

    private val _uiSideEffect = Channel<DeviceListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState =
        combine(deviceRepository.devices, _loadingDevices, _listDevicesError) {
                devices,
                loadingDevices,
                listDevicesError ->
                when {
                    listDevicesError != null -> DeviceListUiState.Error(listDevicesError)
                    devices != null ->
                        DeviceListUiState.Content(
                            devices
                                .sortedBy { it.created.parseAsDateTime() }
                                .map { device ->
                                    DeviceListItemUiState(
                                        device = device,
                                        isLoading =
                                            loadingDevices.any { loadingDevice ->
                                                device.id == loadingDevice
                                            }
                                    )
                                }
                        )
                    else -> DeviceListUiState.Loading
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.Loading)

    init {
        viewModelScope.launch { withContext(dispatcher) { refreshDeviceList(accountToken) } }
    }

    fun removeDevice(accountToken: String, deviceIdToRemove: DeviceId) {
        viewModelScope.launch {
            withContext(dispatcher) {
                Either.catch {
                        withTimeout(DEVICE_REMOVAL_TIMEOUT_MILLIS) {
                            setLoadingDevice(deviceIdToRemove)
                            // Tell the daemon to remove the device
                            deviceRepository.removeDevice(accountToken, deviceIdToRemove)
                            // Wait for the device to be removed
                            deviceRepository.deviceRemovalEvent.first { event ->
                                event.newDevices.none { it.id == deviceIdToRemove }
                            }
                        }
                    }
                    .fold(
                        {
                            clearLoadingDevice(deviceIdToRemove)
                            refreshDeviceList(accountToken)
                            _uiSideEffect.send(DeviceListSideEffect.FailedToRemoveDevice)
                        },
                        { clearLoadingDevice(deviceIdToRemove) }
                    )
            }
        }
    }

    suspend fun refreshDeviceList(accountToken: String) {
        deviceRepository.refreshDeviceList(accountToken)?.let {
            _listDevicesError.value = it
            _uiSideEffect.send(DeviceListSideEffect.FailedToGetDeviceList)
        }
    }

    private fun setLoadingDevice(deviceId: DeviceId) {
        _loadingDevices.value = _loadingDevices.value.toMutableList().apply { add(deviceId) }
    }

    private fun clearLoadingDevice(deviceId: DeviceId) {
        _loadingDevices.value = _loadingDevices.value.toMutableList().apply { remove(deviceId) }
    }

    companion object {
        private const val DEVICE_REMOVAL_TIMEOUT_MILLIS = 5000L
    }
}

sealed interface DeviceListSideEffect {
    data object FailedToRemoveDevice : DeviceListSideEffect

    data object FailedToGetDeviceList : DeviceListSideEffect
}
