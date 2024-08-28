package net.mullvad.mullvadvpn.compose.preview

import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import org.joda.time.DateTime

internal object DevicePreviewData {
    fun generateDevices(count: Int) =
        List(count) { index -> generateDevice(index) }
            .mapIndexed { index, device ->
                DeviceItemUiState(device = device, isLoading = index == 0)
            }

    fun generateDevice(index: Int = 0, id: String = UUID, name: String? = null) =
        Device(
            id = DeviceId.fromString(id),
            name = name ?: "Device $index-${id.take(DEVICE_SUFFIX_LENGTH)}",
            creationDate = DEVICE_CREATION_DATE.plusMonths(index),
        )
}

private const val DEVICE_SUFFIX_LENGTH = 4
private const val UUID = "12345678-1234-5678-1234-567812345678"
private val DEVICE_CREATION_DATE = DateTime.parse("2024-05-27")
