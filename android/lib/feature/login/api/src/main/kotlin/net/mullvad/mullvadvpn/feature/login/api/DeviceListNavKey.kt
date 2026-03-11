package net.mullvad.mullvadvpn.feature.login.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.lib.model.AccountNumber

@Serializable data class DeviceListNavKey(val accountNumber: AccountNumber) : NavKey
