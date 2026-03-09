package net.mullvad.mullvadvpn.feature.login.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.AccountNumber

@Parcelize data class DeviceListNavKey(val accountNumber: AccountNumber) : NavKey2
