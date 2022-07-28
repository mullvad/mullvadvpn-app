package net.mullvad.mullvadvpn.model

sealed class DeviceList {
    object Unavailable : DeviceList()
    data class Available(val devices: List<Device>) : DeviceList()
    object Error : DeviceList()
}
