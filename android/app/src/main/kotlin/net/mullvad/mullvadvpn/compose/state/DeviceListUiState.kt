package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Device

data class DeviceListUiState(
    val deviceUiItems: List<DeviceListItemUiState>,
    val isLoading: Boolean,
    val stagedDevice: Device?
) {
    val hasTooManyDevices = deviceUiItems.count() >= 5

    companion object {
        val INITIAL =
            DeviceListUiState(deviceUiItems = emptyList(), isLoading = true, stagedDevice = null)
    }
}

data class DeviceListItemUiState(val device: Device, val isLoading: Boolean)
