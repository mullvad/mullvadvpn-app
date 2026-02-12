package net.mullvad.mullvadvpn.feature.managedevices.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError

class ManageDevicesUiStatePreviewParameterProvider :
    PreviewParameterProvider<Lce<Unit, ManageDevicesUiState, GetDeviceListError>> {
    override val values =
        sequenceOf(
            Lce.Content(ManageDevicesUiState(generateDevices(NUMBER_OF_DEVICES_NORMAL))),
            Lce.Content(ManageDevicesUiState(generateDevices(NUMBER_OF_DEVICES_TOO_MANY))),
            Lce.Content(ManageDevicesUiState(emptyList())),
            Lce.Loading(Unit),
            Lce.Error(GetDeviceListError.Unknown(IllegalStateException("Error"))),
        )

    private fun generateDevices(count: Int) =
        List(count) { index -> generateDevice(index) }
            .mapIndexed { index, device ->
                ManageDevicesItemUiState(
                    device = device,
                    isLoading = index == 1,
                    isCurrentDevice = index == 0,
                )
            }

    private fun generateDevice(index: Int = 0, id: String = UUID, name: String? = null) =
        Device(
            id = DeviceId.fromString(id),
            name = name ?: "Device $index-${id.take(DEVICE_SUFFIX_LENGTH)}",
            creationDate = DEVICE_CREATION_DATE.plusMonths(index.toLong()),
        )

    companion object {
        private const val NUMBER_OF_DEVICES_NORMAL = 4
        private const val NUMBER_OF_DEVICES_TOO_MANY = 5
        private const val DEVICE_SUFFIX_LENGTH = 4
        private const val UUID = "12345678-1234-5678-1234-567812345678"
        private val DEVICE_CREATION_DATE =
            ZonedDateTime.parse("2024-05-27T00:00+00:00", DateTimeFormatter.ISO_ZONED_DATE_TIME)
    }
}
