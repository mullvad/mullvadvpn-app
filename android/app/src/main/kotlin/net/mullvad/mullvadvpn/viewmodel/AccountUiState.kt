package net.mullvad.mullvadvpn.viewmodel

import net.mullvad.mullvadvpn.model.AccountExpiry
import net.mullvad.mullvadvpn.model.DeviceState
import org.joda.time.DateTime

data class AccountUiState(
    val deviceName: String?,
    val accountNumber: String?,
    val accountExpiry: DateTime?
) {
    companion object {
        fun default() =
            AccountUiState(
                deviceName = DeviceState.Unknown.deviceName(),
                accountNumber = DeviceState.Unknown.token(),
                accountExpiry = AccountExpiry.Missing.date()
            )
    }
}
