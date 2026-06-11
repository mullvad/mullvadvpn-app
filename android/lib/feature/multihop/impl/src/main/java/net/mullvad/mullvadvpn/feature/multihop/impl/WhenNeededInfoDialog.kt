package net.mullvad.mullvadvpn.feature.multihop.impl

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.compose.DescribedIcon
import net.mullvad.mullvadvpn.lib.common.compose.stringResourceWithIcons
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewWhenNeededInfoDialog() {
    AppTheme { WhenNeededInfo(EmptyNavigator) }
}

@Composable
fun WhenNeededInfo(navigator: Navigator) {
    val message =
        stringResourceWithIcons(
            id = R.string.multihop_when_needed_info_first_paragraph,
            DescribedIcon(
                icon = MultihopWhenNeeded,
                contentDescription = stringResource(R.string.multihop_when_needed),
            ),
        )

    InfoDialog(
        title = stringResource(R.string.when_needed),
        message = message.text,
        messageInlineContent = message.inlineContent,
        additionalInfo = stringResource(R.string.automatic_entry_warning),
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
