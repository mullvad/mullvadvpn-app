package net.mullvad.mullvadvpn.feature.apiaccess.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethod
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodName

@Parcelize
data class SaveApiAccessMethodNavKey(
    val id: ApiAccessMethodId?,
    val name: ApiAccessMethodName,
    val customProxy: ApiAccessMethod.CustomProxy,
) : NavKey2

@Parcelize data class SaveApiAccessMethodNavResult(val success: Boolean) : NavResult
