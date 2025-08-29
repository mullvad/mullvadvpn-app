package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.destinations.DeviceListDestination
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
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository

class DeviceListViewModel(
    private val deviceRepository: DeviceRepository,
    savedStateHandle: SavedStateHandle,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {
    private val accountNumber: AccountNumber =
        DeviceListDestination.argsFrom(savedStateHandle).accountNumber

    private val loadingDevices = MutableStateFlow<Set<DeviceId>>(emptySet())
    private val deviceList = MutableStateFlow<List<Device>>(emptyList())
    private val loading = MutableStateFlow(true)
    private val error = MutableStateFlow<GetDeviceListError?>(null)

    private val _uiSideEffect = Channel<DeviceListSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<DeviceListUiState> =
        combine(
                loadingDevices,
                deviceList.map { it.sortedBy { it.creationDate } },
                loading,
                error,
            ) { loadingDevices, devices, loading, error ->
                when {
                    loading -> DeviceListUiState.Loading
                    error != null -> DeviceListUiState.Error(error)
                    else ->
                        DeviceListUiState.Content(
                            devices.map { DeviceItemUiState(it, loadingDevices.contains(it.id)) }
                        )
                }
            }
            .onStart { fetchDevices() }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), DeviceListUiState.Loading)

    fun fetchDevices() =
        viewModelScope.launch {
            error.value = null
            loading.value = true
            deviceRepository
                .deviceList(accountNumber)
                .fold({ error.value = it }, { deviceList.value = it })
            loading.value = false
        }

    fun removeDevice(deviceIdToRemove: DeviceId) =
        viewModelScope.launch(dispatcher) {
            setLoadingState(deviceIdToRemove, true)
            deviceRepository
                .removeDevice(accountNumber, deviceIdToRemove)
                .fold(
                    {
                        _uiSideEffect.send(DeviceListSideEffect.FailedToRemoveDevice)
                        setLoadingState(deviceIdToRemove, false)
                        deviceRepository.deviceList(accountNumber).onRight { deviceList.value = it }
                    },
                    { removeDeviceFromState(deviceIdToRemove) },
                )
        }

    private fun setLoadingState(deviceId: DeviceId, isLoading: Boolean) {
        loadingDevices.update { if (isLoading) it + deviceId else it - deviceId }
    }

    fun continueToLogin() =
        viewModelScope.launch {
            _uiSideEffect.send(DeviceListSideEffect.NavigateToLogin(accountNumber = accountNumber))
        }

    private fun removeDeviceFromState(deviceId: DeviceId) {
        deviceList.update { devices -> devices.filter { item -> item.id != deviceId } }
        loadingDevices.update { it - deviceId }
    }
}

sealed interface DeviceListSideEffect {
    data object FailedToRemoveDevice : DeviceListSideEffect

    data class NavigateToLogin(val accountNumber: AccountNumber) : DeviceListSideEffect
}
