package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun ContentBlockersInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.dns_content_blockers_info),
        additionalInfo = stringResource(id = R.string.dns_content_blockers_warning),
        onDismiss = onDismiss
    )
}
