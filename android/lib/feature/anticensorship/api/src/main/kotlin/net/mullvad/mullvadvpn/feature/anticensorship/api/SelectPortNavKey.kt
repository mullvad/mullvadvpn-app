package net.mullvad.mullvadvpn.feature.anticensorship.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.lib.model.PortType

@Parcelize data class SelectPortNavKey(val portType: PortType) : NavKey2
