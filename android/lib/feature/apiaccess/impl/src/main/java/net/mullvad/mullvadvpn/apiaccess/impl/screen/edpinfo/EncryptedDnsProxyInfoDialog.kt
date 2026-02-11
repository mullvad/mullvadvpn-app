package net.mullvad.mullvadvpn.apiaccess.impl.screen.edpinfo

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.EmptyDestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewEncryptedDnsProxyInfo() {
    AppTheme { EncryptedDnsProxyInfo(EmptyDestinationsNavigator) }
}

@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun EncryptedDnsProxyInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.encrypted_dns_proxy_info_message_part1))
                appendLine()
                appendLine(stringResource(id = R.string.encrypted_dns_proxy_info_message_part2))
            },
        onDismiss = navigator::navigateUp,
    )
}
