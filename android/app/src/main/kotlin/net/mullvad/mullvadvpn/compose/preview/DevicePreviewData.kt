package net.mullvad.mullvadvpn.compose.preview

import java.time.ZonedDateTime
import java.time.format.DateTimeFormatter
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId

internal object DevicePreviewData {
    fun generateDevices(count: Int) =
        List(count) { index -> generateDevice(index) }
            .mapIndexed { index, device ->
                DeviceItemUiState(device = device, isLoading = index == 1)
            }

    fun generateDevice(index: Int = 0, id: String = UUID, name: String? = null) =
        Device(
            id = DeviceId.fromString(id),
            name = name ?: "Device $index-${id.take(DEVICE_SUFFIX_LENGTH)}",
            creationDate = DEVICE_CREATION_DATE.plusMonths(index.toLong()),
        )
}

private const val DEVICE_SUFFIX_LENGTH = 4
private const val UUID = "12345678-1234-5678-1234-567812345678"
private val DEVICE_CREATION_DATE =
    ZonedDateTime.parse("2024-05-27T00:00+00:00", DateTimeFormatter.ISO_ZONED_DATE_TIME)
