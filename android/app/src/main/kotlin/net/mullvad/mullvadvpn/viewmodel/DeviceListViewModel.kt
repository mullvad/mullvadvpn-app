package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onSubscription
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.DeviceListItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceList
import net.mullvad.mullvadvpn.model.RemoveDeviceResult
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository

typealias DeviceId = String

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository,
    private val resources: Resources,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default
) : ViewModel() {
    private val _stagedDeviceId = MutableStateFlow<DeviceId?>(null)

    private val _toastMessages = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val toastMessages = _toastMessages.asSharedFlow()

    var accountToken: String? = null
    private var cachedDeviceList: List<Device>? = null

    val uiState = deviceRepository.deviceList
        .combine(_stagedDeviceId) { deviceList, stagedDeviceId ->
            val devices = if (deviceList is DeviceList.Available) {
                deviceList.devices.also { cachedDeviceList = it }
            } else {
                cachedDeviceList
            }
            val deviceUiItems = devices?.map { device ->
                DeviceListItemUiState(device, false)
            } ?: emptyList()
            val isLoading = devices == null
            val stagedDevice = devices?.firstOrNull { device ->
                device.id == stagedDeviceId
            }
            DeviceListUiState(
                deviceUiItems = deviceUiItems,
                isLoading = isLoading,
                stagedDevice = stagedDevice
            )
        }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.INITIAL)

    fun stageDeviceForRemoval(deviceId: DeviceId) {
        _stagedDeviceId.value = deviceId
    }

    fun clearStagedDevice() {
        _stagedDeviceId.value = null
    }

    fun confirmRemovalOfStagedDevice() {
        val token = accountToken
        val stagedDeviceId = _stagedDeviceId.value

        if (token != null && stagedDeviceId != null) {
            viewModelScope.launch {
                withContext(dispatcher) {
                    val result = withTimeoutOrNull(DEVICE_REMOVAL_TIMEOUT_MILLIS) {
                        deviceRepository.deviceRemovalEvent
                            .onSubscription {
                                clearStagedDevice()
                                deviceRepository.removeDevice(token, stagedDeviceId)
                            }
                            .filter { (deviceId, result) ->
                                deviceId == stagedDeviceId && result == RemoveDeviceResult.Ok
                            }
                            .first()
                    }

                    if (result == null) {
                        _toastMessages.tryEmit(
                            resources.getString(R.string.failed_to_remove_device)
                        )
                        refreshDeviceList()
                    }
                }
            }
        } else {
            _toastMessages.tryEmit(resources.getString(R.string.error_occurred))
            clearStagedDevice()
            refreshDeviceList()
        }
    }

    fun refreshDeviceState() = deviceRepository.refreshDeviceState()

    fun refreshDeviceList() = accountToken?.let { token ->
        deviceRepository.refreshDeviceList(token)
    }

    companion object {
        private const val DEVICE_REMOVAL_TIMEOUT_MILLIS = 5000L
    }
}
