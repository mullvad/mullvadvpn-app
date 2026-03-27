package net.mullvad.mullvadvpn.feature.apiaccess.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId

@Parcelize
data class EditApiAccessMethodNavKey(val accessMethodId: ApiAccessMethodId? = null) : NavKey2

@Parcelize data class EditApiAccessMethodNavResult(val success: Boolean) : NavResult
