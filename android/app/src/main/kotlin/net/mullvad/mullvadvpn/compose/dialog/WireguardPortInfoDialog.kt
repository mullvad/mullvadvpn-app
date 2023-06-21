package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.model.PortRange

@Composable
fun WireguardPortInfoDialog(portRanges: List<PortRange>, onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.wireguard_port_info_description),
        additionalInfo =
            buildString {
                portRanges.forEachIndexed { index, range ->
                    if (index != 0) {
                        append(",")
                        append(" ")
                    }
                    if (range.from == range.to) {
                        append(range.from)
                    } else {
                        append("${range.from}-${range.to}")
                    }
                }
            },
        onDismiss = onDismiss
    )
}
