package net.mullvad.mullvadvpn.feature.managedevices.impl

class ManageDeviceComparator : Comparator<ManageDevicesItemUiState> {
    override fun compare(
        deviceItem: ManageDevicesItemUiState?,
        otherDeviceItem: ManageDevicesItemUiState?,
    ): Int =
        when {
            deviceItem == null -> 1
            otherDeviceItem == null -> -1
            deviceItem.isCurrentDevice -> -1
            otherDeviceItem.isCurrentDevice -> 1
            else -> deviceItem.device.creationDate.compareTo(otherDeviceItem.device.creationDate)
        }
}
