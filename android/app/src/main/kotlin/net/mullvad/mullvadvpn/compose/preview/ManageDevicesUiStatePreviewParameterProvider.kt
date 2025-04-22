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
                    DevicePreviewData.generateDevices(NUMBER_OF_DEVICES_NORMAL)
                        .toManageDevicesState()
                )
            ),
            Lce.Content(
                ManageDevicesUiState(
                    DevicePreviewData.generateDevices(NUMBER_OF_DEVICES_TOO_MANY)
                        .toManageDevicesState()
                )
            ),
            Lce.Content(ManageDevicesUiState(emptyList())),
            Lce.Loading,
            Lce.Error(GetDeviceListError.Unknown(IllegalStateException("Error"))),
        )

    private fun List<DeviceItemUiState>.toManageDevicesState() = mapIndexed { index, state ->
        ManageDevicesItemUiState(
            device = state.device,
            isLoading = state.isLoading,
            isCurrentDevice = index == 0,
        )
    }
}

private const val NUMBER_OF_DEVICES_NORMAL = 4
private const val NUMBER_OF_DEVICES_TOO_MANY = 5
