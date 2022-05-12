package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize
    object InitialState : DeviceState()

    @Parcelize
    class LoggedIn(val accountAndDevice: AccountAndDevice) : DeviceState()

    @Parcelize
    object LoggedOut : DeviceState()

    @Parcelize
    object Revoked : DeviceState()

    fun isInitialState(): Boolean {
        return this is InitialState
    }

    fun deviceName(): String? {
        return (this as? LoggedIn)?.accountAndDevice?.device?.name
    }

    fun token(): String? {
        return (this as? LoggedIn)?.accountAndDevice?.account_token
    }
}
