package net.mullvad.mullvadvpn.viewmodel

import android.content.res.Resources
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.onSubscription
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.DeviceListItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.common.util.parseAsDateTime
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceList
import net.mullvad.mullvadvpn.model.RemoveDeviceResult
import net.mullvad.mullvadvpn.repository.DeviceRepository

typealias DeviceId = String

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository,
    private val resources: Resources,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default
) : ViewModel() {
    private val _loadingDevices = MutableStateFlow<List<DeviceId>>(emptyList())

    private val _uiSideEffect = Channel<DeviceListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    private var cachedDeviceList: List<Device>? = null

    val uiState =
        combine(deviceRepository.deviceList, _loadingDevices) { deviceList, loadingDevices ->
                val devices =
                    if (deviceList is DeviceList.Available) {
                        deviceList.devices.also { cachedDeviceList = it }
                    } else {
                        cachedDeviceList
                    }
                val deviceUiItems =
                    devices
                        ?.sortedBy { it.created.parseAsDateTime() }
                        ?.map { device ->
                            DeviceListItemUiState(
                                device,
                                loadingDevices.any { loadingDevice -> device.id == loadingDevice }
                            )
                        }
                val isLoading = devices == null
                DeviceListUiState(
                    deviceUiItems = deviceUiItems ?: emptyList(),
                    isLoading = isLoading,
                )
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.INITIAL)

    fun removeDevice(accountToken: String, deviceIdToRemove: DeviceId) {

        viewModelScope.launch {
            withContext(dispatcher) {
                val result =
                    withTimeoutOrNull(DEVICE_REMOVAL_TIMEOUT_MILLIS) {
                        deviceRepository.deviceRemovalEvent
                            .onSubscription {
                                setLoadingDevice(deviceIdToRemove)
                                deviceRepository.removeDevice(accountToken, deviceIdToRemove)
                            }
                            .filter { (deviceId, result) ->
                                deviceId == deviceIdToRemove && result == RemoveDeviceResult.Ok
                            }
                            .first()
                    }

                clearLoadingDevice(deviceIdToRemove)

                if (result == null) {
                    _uiSideEffect.send(
                        DeviceListSideEffect.ShowToast(
                            resources.getString(R.string.failed_to_remove_device)
                        )
                    )
                    refreshDeviceList(accountToken)
                }
            }
        }
    }

    fun refreshDeviceState() = deviceRepository.refreshDeviceState()

    fun refreshDeviceList(accountToken: String) = deviceRepository.refreshDeviceList(accountToken)

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
    data class ShowToast(val text: String) : DeviceListSideEffect
}
