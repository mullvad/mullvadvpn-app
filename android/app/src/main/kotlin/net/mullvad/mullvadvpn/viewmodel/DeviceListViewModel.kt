package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.util.safeLet

typealias DeviceId = String

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository
) : ViewModel() {
    private val _stagedDeviceId = MutableStateFlow<DeviceId?>(null)

    private val _toastMessages = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val toastMessages = _toastMessages.asSharedFlow()

    var accountToken: String? = null

    val uiState = deviceRepository.deviceList
        .combine(_stagedDeviceId) { deviceList, stagedDeviceId ->
            val stagedDevice = deviceList.firstOrNull { device ->
                device.id == stagedDeviceId
            }
            DeviceListUiState(
                devices = deviceList,
                isLoading = false,
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
        safeLet(accountToken, _stagedDeviceId.value) { token, deviceId ->
            deviceRepository.removeDevice(token, deviceId)
            _stagedDeviceId.value = null
        }
    }

    fun refreshDeviceState() = deviceRepository.refreshDeviceState()

    fun refreshDeviceList() = accountToken?.let { token ->
        deviceRepository.refreshDeviceList(token)
    }
}
