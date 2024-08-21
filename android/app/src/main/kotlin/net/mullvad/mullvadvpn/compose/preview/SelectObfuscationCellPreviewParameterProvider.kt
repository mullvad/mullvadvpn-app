package net.mullvad.mullvadvpn.compose.preview

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.SelectedObfuscation

class SelectObfuscationCellPreviewParameterProvider :
    PreviewParameterProvider<Triple<SelectedObfuscation, Constraint<Port>?, Boolean>> {
    override val values: Sequence<Triple<SelectedObfuscation, Constraint<Port>?, Boolean>> =
        sequenceOf(
            Triple(SelectedObfuscation.Auto, null, false),
            Triple(SelectedObfuscation.Auto, null, true),
            Triple(SelectedObfuscation.Shadowsocks, Constraint.Any, false),
            Triple(SelectedObfuscation.Shadowsocks, Constraint.Any, true),
            Triple(SelectedObfuscation.Shadowsocks, Constraint.Only(Port(PORT)), false),
            Triple(SelectedObfuscation.Shadowsocks, Constraint.Only(Port(PORT)), true),
            Triple(SelectedObfuscation.Udp2Tcp, Constraint.Any, false),
            Triple(SelectedObfuscation.Udp2Tcp, Constraint.Any, true),
            Triple(SelectedObfuscation.Udp2Tcp, Constraint.Only(Port(PORT)), false),
            Triple(SelectedObfuscation.Udp2Tcp, Constraint.Only(Port(PORT)), true),
            Triple(SelectedObfuscation.Off, null, false),
            Triple(SelectedObfuscation.Off, null, true),
        )
}

private const val PORT = 44
