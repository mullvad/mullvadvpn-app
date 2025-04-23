package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.compose.state.ManageDevicesItemUiState
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.shared.DeviceRepository
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.util.toLce

class ManageDevicesViewModel(
    deviceRepository: DeviceRepository,
    private val deviceListViewModel: DeviceListViewModel,
) : ViewModel() {

    val uiSideEffect =
        deviceListViewModel.uiSideEffect
            .filter { it is DeviceListSideEffect.FailedToRemoveDevice }
            .map { ManageDevicesSideEffect.FailedToRemoveDevice }

    val uiState: StateFlow<Lce<ManageDevicesUiState, GetDeviceListError>> =
        combine(
                deviceRepository.deviceState.filterIsInstance<DeviceState.LoggedIn>(),
                deviceListViewModel.uiState,
            ) { loggedInState, deviceListState ->
                when (deviceListState) {
                    DeviceListUiState.Loading -> Lce.Loading
                    is DeviceListUiState.Error -> Lce.Error(deviceListState.error)
                    is DeviceListUiState.Content -> {
                        ManageDevicesUiState(
                                deviceListState.devices.toManageDevicesItemUiState(
                                    currentDevice = loggedInState.device
                                )
                            )
                            .toLce()
                    }
                }
            }
            .stateIn(viewModelScope, SharingStarted.WhileSubscribed(), Lce.Loading)

    fun fetchDevices() = deviceListViewModel.fetchDevices()

    fun removeDevice(deviceIdToRemove: DeviceId) =
        deviceListViewModel.removeDevice(deviceIdToRemove)

    private fun List<DeviceItemUiState>.toManageDevicesItemUiState(
        currentDevice: Device
    ): List<ManageDevicesItemUiState> {
        // Put the current device first in the list, but otherwise keep the sort order.
        val devices = toMutableList()
        devices
            .indexOfFirst { it.device.id == currentDevice.id }
            .let { index ->
                if (index > 0) {
                    devices.add(0, devices.removeAt(index))
                }
            }

        return devices.map {
            ManageDevicesItemUiState(
                device = it.device,
                isLoading = it.isLoading,
                isCurrentDevice = it.device.id == currentDevice.id,
            )
        }
    }
}

sealed interface ManageDevicesSideEffect {
    data object FailedToRemoveDevice : ManageDevicesSideEffect
}
