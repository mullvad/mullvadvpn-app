package net.mullvad.mullvadvpn.feature.apiaccess.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.nav3.NavKey2
import net.mullvad.mullvadvpn.lib.model.ApiAccessMethodId

@Parcelize
data class EditApiAccessMethodNavKey(val accessMethodId: ApiAccessMethodId? = null) : NavKey2


