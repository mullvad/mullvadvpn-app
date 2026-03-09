package net.mullvad.mullvadvpn.feature.managedevices.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.AccountNumber

@Parcelize data class ManageDevicesNavKey(val accountNumber: AccountNumber) : NavKey2
