package net.mullvad.mullvadvpn.feature.multihop.impl

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
private fun PreviewWhenNeededInfoDialog() {
    AppTheme { WhenNeededInfo(EmptyNavigator) }
}

@Composable
fun WhenNeededInfo(navigator: Navigator) {
    InfoDialog(
        title = stringResource(R.string.when_needed),
        message = stringResource(R.string.multihop_when_needed_info_first_paragraph),
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
