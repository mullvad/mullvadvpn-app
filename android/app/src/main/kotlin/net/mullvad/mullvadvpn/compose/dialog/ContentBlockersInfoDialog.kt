package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun ContentBlockersInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.dns_content_blockers_info))
                append(stringResource(id = R.string.dns_content_blockers_warning))
            },
        additionalInfo =
            buildString {
                appendLine(textResource(id = R.string.dns_content_blockers_custom_dns_warning))
                appendLine(
                    stringResource(id = R.string.settings_changes_effect_warning_content_blocker)
                )
            },
        onDismiss = navigator::navigateUp
    )
}
