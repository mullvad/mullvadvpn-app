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
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.ui.serviceconnection.DeviceRepository
import net.mullvad.mullvadvpn.util.safeLet

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository
) : ViewModel() {
    private val _stagedForRemoval = MutableStateFlow<Device?>(null)

    private val _toastMessages = MutableSharedFlow<String>(extraBufferCapacity = 1)
    val toastMessages = _toastMessages.asSharedFlow()

    var accountToken: String? = null

    val uiState = deviceRepository.deviceList
        .combine(_stagedForRemoval) { deviceList, deviceStagedForRemoval ->
            DeviceListUiState(
                devices = deviceList,
                isLoading = false,
                deviceStagedForRemoval = deviceStagedForRemoval
            )
        }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.INITIAL)

    fun stageDeviceForRemoval(device: Device) {
        _stagedForRemoval.value = device
    }

    fun clearStagedDevice() {
        _stagedForRemoval.value = null
    }

    fun confirmRemoval() {
        safeLet(accountToken, _stagedForRemoval.value) { token, device ->
            deviceRepository.removeDevice(token, device.id)
            _stagedForRemoval.value = null
        }
    }

    fun refreshDeviceState() = deviceRepository.refreshDeviceState()

    fun refreshDeviceList() = accountToken?.let { token ->
        deviceRepository.refreshDeviceList(token)
    }
}
