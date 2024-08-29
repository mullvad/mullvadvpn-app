package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.compose.preview.DevicePreviewData.generateDevices
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState

class DeviceListPreviewParameterProvider : PreviewParameterProvider<List<DeviceItemUiState>> {
    override val values =
        sequenceOf(
            generateDevices(NUMBER_OF_DEVICES_NORMAL),
            generateDevices(NUMBER_OF_DEVICES_TOO_MANY),
        )
}

private const val NUMBER_OF_DEVICES_NORMAL = 4
private const val NUMBER_OF_DEVICES_TOO_MANY = 5
