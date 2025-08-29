package net.mullvad.mullvadvpn.compose.state

import net.mullvad.mullvadvpn.lib.model.Device

data class ManageDevicesUiState(val devices: List<ManageDevicesItemUiState>)

data class ManageDevicesItemUiState(
    val device: Device,
    val isLoading: Boolean,
    val isCurrentDevice: Boolean,
)
