package net.mullvad.mullvadvpn.feature.dns.impl.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun ContentBlockersInfo(navigator: Navigator) {
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
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
