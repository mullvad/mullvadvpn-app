package net.mullvad.mullvadvpn.feature.apiaccess.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId

@Parcelize data class ApiAccessMethodDetailsNavKey(val accessMethodId: ApiAccessMethodId) : NavKey2
