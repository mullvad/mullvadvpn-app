package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource

@Composable
fun CustomDnsInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message =
        stringResource(id = R.string.settings_changes_effect_warning_custom_dns),
        onDismiss = onDismiss,
    )
}
