package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.model.AccountToken
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId
import net.mullvad.mullvadvpn.model.GetDeviceListError
import net.mullvad.mullvadvpn.repository.DeviceRepository

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository,
    private val token: AccountToken,
    private val dispatcher: CoroutineDispatcher = Dispatchers.Default,
) : ViewModel() {
    private val loadingDevices = MutableStateFlow<List<DeviceId>>(emptyList())
    private val deviceList = MutableStateFlow<List<Device>>(emptyList())
    private val loading = MutableStateFlow(true)
    private val error = MutableStateFlow<GetDeviceListError?>(null)

    private val _uiSideEffect = Channel<DeviceListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<DeviceListUiState> =
        combine(loadingDevices, deviceList.map { it.sortedBy { it.created } }, loading, error) {
                loadingDevices,
                devices,
                loading,
                error ->
                when {
                    loading -> DeviceListUiState.Loading
                    error != null -> DeviceListUiState.Error(error)
                    else -> DeviceListUiState.Content(devices, loadingDevices)
                }
            }
            .onStart { fetchDevices() }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.Loading)

    fun fetchDevices() =
        viewModelScope.launch {
            error.value = null
            loading.value = true
            deviceRepository.deviceList(token).fold({ error.value = it }, { deviceList.value = it })
            loading.value = false
        }

    fun removeDevice(deviceIdToRemove: DeviceId) =
        viewModelScope.launch(dispatcher) {
            setLoadingState(deviceIdToRemove, true)
            deviceRepository
                .removeDevice(token, deviceIdToRemove)
                .fold(
                    {
                        _uiSideEffect.send(DeviceListSideEffect.FailedToRemoveDevice)
                        setLoadingState(deviceIdToRemove, false)
                        deviceRepository.deviceList(token).onRight { deviceList.value = it }
                    },
                    { removeDeviceFromState(deviceIdToRemove) }
                )
        }

    private fun setLoadingState(deviceId: DeviceId, isLoading: Boolean) {
        loadingDevices.update { if (isLoading) it + deviceId else it - deviceId }
    }

    private fun removeDeviceFromState(deviceId: DeviceId) {
        deviceList.update { devices -> devices.filter { item -> item.id != deviceId } }
        loadingDevices.update { it - deviceId }
    }
}

sealed interface DeviceListSideEffect {
    data object FailedToRemoveDevice : DeviceListSideEffect
}
