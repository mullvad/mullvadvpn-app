package net.mullvad.mullvadvpn.feature.login.impl.devicelist

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.feature.login.impl.devicelist.DevicePreviewData.generateDevices
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError

class DeviceListUiStatePreviewParameterProvider : PreviewParameterProvider<DeviceListUiState> {
    override val values =
        sequenceOf(
            DeviceListUiState.Content(devices = generateDevices(NUMBER_OF_DEVICES_NORMAL)),
            DeviceListUiState.Content(devices = generateDevices(NUMBER_OF_DEVICES_TOO_MANY)),
            DeviceListUiState.Content(devices = emptyList()),
            DeviceListUiState.Loading,
            DeviceListUiState.Error(GetDeviceListError.Unknown(IllegalStateException("Error"))),
        )
}

private const val NUMBER_OF_DEVICES_NORMAL = 4
private const val NUMBER_OF_DEVICES_TOO_MANY = 5
