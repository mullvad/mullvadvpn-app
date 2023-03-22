package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceListEvent : Parcelable {
    @Parcelize
    data class Available(val accountToken: String, val devices: List<Device>) : DeviceListEvent()

    @Parcelize object Error : DeviceListEvent()

    fun isAvailable(): Boolean {
        return (this is Available)
    }
}
