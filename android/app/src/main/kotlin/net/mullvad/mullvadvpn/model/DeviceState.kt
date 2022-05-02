package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize
    object InitialState : DeviceState()

    @Parcelize
    data class DeviceRegistered(val deviceConfig: AccountAndDevice) : DeviceState()

    @Parcelize
    object DeviceNotRegistered : DeviceState()

    fun isInitialState(): Boolean {
        return this is InitialState
    }

    fun deviceName(): String? {
        return (this as? DeviceRegistered)?.deviceConfig?.device?.name
    }

    fun token(): String? {
        return (this as? DeviceRegistered)?.deviceConfig?.account_token
    }

    companion object {
        fun fromDeviceConfig(deviceConfig: AccountAndDevice?): DeviceState {
            return deviceConfig?.let { DeviceRegistered(it) } ?: DeviceNotRegistered
        }
    }
}
