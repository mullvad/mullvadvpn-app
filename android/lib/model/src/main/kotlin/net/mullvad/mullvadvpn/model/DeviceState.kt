package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize object Initial : DeviceState()

    @Parcelize object Unknown : DeviceState()

    @Parcelize data class LoggedIn(val accountToken: String, val device: Device) : DeviceState()

    @Parcelize object LoggedOut : DeviceState()

    @Parcelize object Revoked : DeviceState()

    fun isUnknown(): Boolean {
        return this is Unknown
    }

    fun deviceName(): String? {
        return (this as? LoggedIn)?.device?.displayName()
    }

    fun token(): String? {
        return (this as? LoggedIn)?.accountToken
    }
}
