package net.mullvad.mullvadvpn.lib.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize
    data class LoggedIn(val accountToken: AccountToken, val device: Device) : DeviceState()

    @Parcelize data object LoggedOut : DeviceState()

    @Parcelize data object Revoked : DeviceState()

    fun displayName(): String? {
        return (this as? LoggedIn)?.device?.displayName()
    }

    fun token(): AccountToken? {
        return (this as? LoggedIn)?.accountToken
    }
}
