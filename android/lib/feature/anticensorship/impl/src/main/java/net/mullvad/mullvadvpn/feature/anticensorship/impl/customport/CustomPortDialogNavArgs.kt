package net.mullvad.mullvadvpn.feature.anticensorship.impl.customport

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.PortType

@Parcelize
data class CustomPortDialogNavArgs(
    val portType: PortType,
    val allowedPortRanges: List<PortRange>,
    val recommendedPortRanges: List<PortRange>,
    val customPort: Port?,
) : Parcelable
