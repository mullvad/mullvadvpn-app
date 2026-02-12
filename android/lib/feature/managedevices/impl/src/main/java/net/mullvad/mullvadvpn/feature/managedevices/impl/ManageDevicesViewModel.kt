package net.mullvad.mullvadvpn.feature.managedevices.impl

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.ramcosta.composedestinations.generated.managedevices.destinations.ManageDevicesDestination
import kotlin.collections.filter
import kotlin.collections.map
import kotlin.collections.plus
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.WhileSubscribed
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.common.constant.VIEW_MODEL_STOP_TIMEOUT
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.repository.DeviceRepository

class ManageDevicesViewModel(
    private val deviceRepository: DeviceRepository,
    private val dispatcher: CoroutineDispatcher = Dispatchers.IO,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val loadingDevices = MutableStateFlow<Set<DeviceId>>(emptySet())
    private val deviceList = MutableStateFlow<List<Device>>(emptyList())
    private val loading = MutableStateFlow(true)
    private val error = MutableStateFlow<GetDeviceListError?>(null)

    private val accountNumber: AccountNumber =
        ManageDevicesDestination.argsFrom(savedStateHandle).accountNumber

    private val deviceComparator = ManageDeviceComparator()
    private val _uiSideEffect = Channel<ManageDevicesSideEffect>()
    val uiSideEffect = _uiSideEffect.receiveAsFlow()

    val uiState: StateFlow<Lce<Unit, ManageDevicesUiState, GetDeviceListError>> =
        combine(
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>(),
                loadingDevices,
                deviceList,
                loading,
                error,
            ) { currentDeviceState, loadingDevices, devices, loading, error ->
                when {
                    loading -> Lce.Loading(Unit)
                    error != null -> Lce.Error(error)
                    else -> {
                        val deviceItems =
                            devices.map {
                                ManageDevicesItemUiState(
                                    it,
                                    loadingDevices.contains(it.id),
                                    isCurrentDevice = it.id == currentDeviceState.device.id,
                                )
                            }
                        Lce.Content(ManageDevicesUiState(deviceItems.sortedWith(deviceComparator)))
                    }
                }
            }
            .onStart { fetchDevices() }
            .stateIn(
                viewModelScope,
                SharingStarted.WhileSubscribed(VIEW_MODEL_STOP_TIMEOUT),
                Lce.Loading(Unit),
            )

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
                        _uiSideEffect.send(ManageDevicesSideEffect.FailedToRemoveDevice)
                        setLoadingState(deviceIdToRemove, false)
                        deviceRepository.deviceList(accountNumber).onRight { deviceList.value = it }
                    },
                    { removeDeviceFromState(deviceIdToRemove) },
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

sealed interface ManageDevicesSideEffect {
    data object FailedToRemoveDevice : ManageDevicesSideEffect
}
