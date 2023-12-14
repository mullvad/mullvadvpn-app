package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewUdpOverTcpPortInfoDialog() {
    AppTheme { UdpOverTcpPortInfoDialog(EmptyDestinationsNavigator) }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun UdpOverTcpPortInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message = stringResource(id = R.string.udp_over_tcp_port_info),
        onDismiss = navigator::navigateUp
    )
}
