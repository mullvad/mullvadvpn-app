package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewCustomDnsInfoDialog() {
    CustomDnsInfoDialog(onDismiss = {})
}

@Composable
fun CustomDnsInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.settings_changes_effect_warning_content_blocker),
        onDismiss = onDismiss
    )
}
