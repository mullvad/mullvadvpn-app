package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.util.asString

@Composable
fun WireguardPortInfoDialog(portRanges: List<PortRange>, onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.wireguard_port_info_description),
        additionalInfo =
            stringResource(id = R.string.wireguard_port_info_port_range, portRanges.asString()),
        onDismiss = onDismiss
    )
}
