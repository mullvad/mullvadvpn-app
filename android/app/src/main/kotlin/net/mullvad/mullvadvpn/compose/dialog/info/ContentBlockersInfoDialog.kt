package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.screen.MainGraph
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog

@Destination<MainGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ContentBlockersInfo(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.dns_content_blockers_info))
                append(stringResource(id = R.string.dns_content_blockers_warning))
            },
        additionalInfo =
            buildString {
                appendLine(stringResource(id = R.string.dns_content_blockers_custom_dns_warning))
                appendLine(
                    stringResource(id = R.string.settings_changes_effect_warning_content_blocker)
                )
            },
        onDismiss = dropUnlessResumed { navigator.navigateUp() },
    )
}
