package net.mullvad.mullvadvpn.feature.login.impl.devicelist

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import arrow.core.compareTo
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository

class DeviceListViewModel(
    private val accountNumber: AccountNumber,
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
) : ViewModel() {

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
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                DeviceListUiState.Loading,
            )

    fun fetchDevices() = viewModelScope.launch {
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

    fun continueToLogin() = viewModelScope.launch {
        if (deviceList.value.size < MAXIMUM_DEVICES) {
            _uiSideEffect.send(DeviceListSideEffect.NavigateToLogin(accountNumber = accountNumber))
        } else {
            _uiSideEffect.send(DeviceListSideEffect.FailedToLogin)
        }
    }

    private fun removeDeviceFromState(deviceId: DeviceId) {
        deviceList.update { devices -> devices.filter { item -> item.id != deviceId } }
        loadingDevices.update { it - deviceId }
    }

    companion object {
        private const val MAXIMUM_DEVICES = 5
    }
}

sealed interface DeviceListSideEffect {
    data object FailedToRemoveDevice : DeviceListSideEffect

    data object FailedToLogin : DeviceListSideEffect

    data class NavigateToLogin(val accountNumber: AccountNumber) : DeviceListSideEffect
}
