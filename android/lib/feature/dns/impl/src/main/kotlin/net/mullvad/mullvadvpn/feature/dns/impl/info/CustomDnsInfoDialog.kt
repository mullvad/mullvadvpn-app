package net.mullvad.mullvadvpn.feature.dns.impl.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewCustomDnsInfoDialog() {
    AppTheme { CustomDnsInfo(EmptyNavigator) }
}

@Composable
fun CustomDnsInfo(navigator: Navigator) {
    InfoDialog(
        message = stringResource(id = R.string.settings_changes_effect_warning_content_blocker),
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
