package net.mullvad.mullvadvpn.feature.managedevices.api

import androidx.navigation3.runtime.NavKey
import kotlinx.parcelize.Parcelize
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.lib.model.AccountNumber

@Parcelize
data class ManageDevicesNavKey(val accountNumber: AccountNumber) : NavKey2
