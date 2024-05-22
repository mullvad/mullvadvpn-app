package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError

sealed interface DeviceListUiState {
    data object Loading : DeviceListUiState

    data class Error(val error: GetDeviceListError) : DeviceListUiState

    data class Content(
        val devices: List<Device>,
        val loadingDevices: List<DeviceId> = emptyList()
    ) : DeviceListUiState {
        val hasTooManyDevices = devices.size >= MAXIMUM_DEVICES
    }

    companion object {
        val INITIAL: DeviceListUiState = Loading
    }
}

private const val MAXIMUM_DEVICES = 5
