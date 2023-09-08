package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource

@Preview
@Composable
private fun PreviewLocalNetworkSharingInfoDialog() {
    LocalNetworkSharingInfoDialog(onDismiss = {})
}

@Composable
fun LocalNetworkSharingInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.local_network_sharing_info),
        additionalInfo =
            buildString {
                appendLine(stringResource(id = R.string.local_network_sharing_additional_info))
                appendLine(textResource(id = R.string.local_network_sharing_ip_ranges))
            },
        onDismiss = onDismiss
    )
}
