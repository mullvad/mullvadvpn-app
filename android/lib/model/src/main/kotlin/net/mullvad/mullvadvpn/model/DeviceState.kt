package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize data object Initial : DeviceState()

    @Parcelize data object Unknown : DeviceState()

    @Parcelize data class LoggedIn(val accountToken: String, val device: Device) : DeviceState()

    @Parcelize data object LoggedOut : DeviceState()

    @Parcelize data object Revoked : DeviceState()

    fun deviceName(): String? {
        return (this as? LoggedIn)?.device?.displayName()
    }

    fun token(): String? {
        return (this as? LoggedIn)?.accountToken
    }
}
