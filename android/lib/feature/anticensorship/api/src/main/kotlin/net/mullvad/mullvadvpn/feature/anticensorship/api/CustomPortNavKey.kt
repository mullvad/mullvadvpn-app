package net.mullvad.mullvadvpn.feature.anticensorship.api

import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.core.NavKey2
import net.mullvad.mullvadvpn.core.NavResult
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType

@Parcelize
data class CustomPortNavKey(
    val portType: PortType,
    val allowedPortRanges: List<PortRange>,
    val recommendedPortRanges: List<PortRange>,
    val customPort: Port?,
) : NavKey2

@Parcelize data class CustomPortNavResult(val port: Port?) : NavResult
