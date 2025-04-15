package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.compose.state.ManageDevicesItemUiState
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.util.Lce

class ManageDevicesUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<ManageDevicesUiState, GetDeviceListError>> {
    override val values =
        sequenceOf(
            Lce.Content(
                ManageDevicesUiState(
                    toManageDevicesState(
                        DevicePreviewData.generateDevices(NUMBER_OF_DEVICES_NORMAL)
                    )
                )
            ),
            Lce.Content(
                ManageDevicesUiState(
                    toManageDevicesState(
                        DevicePreviewData.generateDevices(NUMBER_OF_DEVICES_TOO_MANY)
                    )
                )
            ),
            Lce.Content(ManageDevicesUiState(emptyList())),
            Lce.Loading,
            Lce.Error(GetDeviceListError.Unknown(IllegalStateException("Error"))),
        )
}

private const val NUMBER_OF_DEVICES_NORMAL = 4
private const val NUMBER_OF_DEVICES_TOO_MANY = 5

private fun toManageDevicesState(items: List<DeviceItemUiState>) =
    items.mapIndexed { index, state ->
        ManageDevicesItemUiState(
            device = state.device,
            isLoading = state.isLoading,
            isCurrentDevice = index == 0,
        )
    }
