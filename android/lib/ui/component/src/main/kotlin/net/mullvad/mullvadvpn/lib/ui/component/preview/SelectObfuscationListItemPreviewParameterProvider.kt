package net.mullvad.mullvadvpn.lib.ui.component.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port

class SelectObfuscationListItemPreviewParameterProvider :
    PreviewParameterProvider<Triple<ObfuscationMode, Constraint<Port>, Boolean>> {
    override val values: Sequence<Triple<ObfuscationMode, Constraint<Port>, Boolean>> =
        sequenceOf(
            Triple(ObfuscationMode.Shadowsocks, Constraint.Any, false),
            Triple(ObfuscationMode.Shadowsocks, Constraint.Any, true),
            Triple(ObfuscationMode.Shadowsocks, Constraint.Only(Port(PORT)), false),
            Triple(ObfuscationMode.Shadowsocks, Constraint.Only(Port(PORT)), true),
            Triple(ObfuscationMode.Udp2Tcp, Constraint.Any, false),
            Triple(ObfuscationMode.Udp2Tcp, Constraint.Any, true),
            Triple(ObfuscationMode.Udp2Tcp, Constraint.Only(Port(PORT)), false),
            Triple(ObfuscationMode.Udp2Tcp, Constraint.Only(Port(PORT)), true),
        )
}

private const val PORT = 44
