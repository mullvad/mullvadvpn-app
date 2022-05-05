package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

sealed class DeviceState : Parcelable {
    @Parcelize
    object InitialState : DeviceState()

    @Parcelize
    data class DeviceRegistered(val accountAndDevice: AccountAndDevice) : DeviceState()

    @Parcelize
    object DeviceNotRegistered : DeviceState()

    fun isInitialState(): Boolean {
        return this is InitialState
    }

    fun deviceName(): String? {
        return (this as? DeviceRegistered)?.accountAndDevice?.device?.name
    }

    fun token(): String? {
        return (this as? DeviceRegistered)?.accountAndDevice?.account_token
    }

    companion object {
        fun from(accountAndDevice: AccountAndDevice?): DeviceState {
            return accountAndDevice?.let { DeviceRegistered(it) } ?: DeviceNotRegistered
        }
    }
}
