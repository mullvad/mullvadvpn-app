package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.ListDevicesError

sealed interface DeviceListUiState {

    data object Loading : DeviceListUiState

    data class Error(val listDevicesError: ListDevicesError) : DeviceListUiState

    data class Content(val deviceUiItems: List<DeviceListItemUiState>) : DeviceListUiState {
        val hasTooManyDevices: Boolean = deviceUiItems.size >= MAXIMUM_DEVICES
    }
}

data class DeviceListItemUiState(val device: Device, val isLoading: Boolean)

private const val MAXIMUM_DEVICES = 5
