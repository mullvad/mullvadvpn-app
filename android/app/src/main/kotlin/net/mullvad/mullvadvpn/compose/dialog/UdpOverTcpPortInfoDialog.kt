package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewUdpOverTcpPortInfoDialog() {
    UdpOverTcpPortInfoDialog(onDismiss = {})
}

@Composable
fun UdpOverTcpPortInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.udp_over_tcp_port_info),
        onDismiss = onDismiss
    )
}
