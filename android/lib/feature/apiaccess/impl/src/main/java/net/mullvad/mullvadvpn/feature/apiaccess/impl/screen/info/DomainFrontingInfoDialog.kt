package net.mullvad.mullvadvpn.feature.apiaccess.impl.screen.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewDomainFrontingInfo() {
    AppTheme { DomainFrontingInfo(EmptyNavigator) }
}

@Composable
fun DomainFrontingInfo(navigator: Navigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(
                    stringResource(
                        id = R.string.domain_fronting_info_message_part1,
                        stringResource(R.string.domain_fronting),
                    )
                )
                appendLine()
                appendLine(stringResource(id = R.string.domain_fronting_info_message_part2))
            },
        onDismiss = navigator::goBack,
    )
}
