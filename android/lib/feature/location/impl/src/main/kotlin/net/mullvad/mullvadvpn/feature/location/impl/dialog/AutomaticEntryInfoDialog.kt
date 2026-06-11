package net.mullvad.mullvadvpn.feature.location.impl.dialog

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
private fun PreviewAutomaticEntryInfoDialog() {
    AppTheme { AutomaticEntryInfo(EmptyNavigator) }
}

@Composable
fun AutomaticEntryInfo(navigator: Navigator) {
    InfoDialog(
        title = stringResource(R.string.automatic),
        message = stringResource(R.string.automatic_entry_info_first_paragraph),
        additionalInfo = stringResource(R.string.automatic_entry_warning),
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
