package net.mullvad.mullvadvpn.feature.customvpn.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2

@Parcelize data class EditCustomTunnelNavKey(val new: Boolean) : NavKey2
