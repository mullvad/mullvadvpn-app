package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize
    object InitialState : DeviceState()

    @Parcelize
    data class DeviceRegistered(val deviceConfig: DeviceConfig) : DeviceState()

    @Parcelize
    object DeviceNotRegistered : DeviceState()

    fun isInitialState(): Boolean {
        return this is InitialState
    }

    fun token(): String? {
        return (this as? DeviceRegistered)?.deviceConfig?.token
    }

    companion object {
        fun fromDeviceConfig(deviceConfig: DeviceConfig?): DeviceState {
            return deviceConfig?.let { DeviceRegistered(it) } ?: DeviceNotRegistered
        }
    }
}
