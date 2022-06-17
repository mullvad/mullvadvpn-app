package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.model.Device

data class DeviceListUiState(
    val devices: List<Device>,
    val isLoading: Boolean,
    val deviceStagedForRemoval: Device?
) {
    val hasTooManyDevices = devices.count() >= 5

    companion object {
        val INITIAL = DeviceListUiState(
            devices = listOf(),
            isLoading = true,
            deviceStagedForRemoval = null
        )
    }
}
