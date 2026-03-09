package net.mullvad.mullvadvpn.feature.vpnsettings.impl.info

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoDialog
import net.mullvad.mullvadvpn.lib.ui.resource.R

@Composable
fun DeviceIpInfo(navigator: Navigator) {
    InfoDialog(
        message =
            buildString {
                append(stringResource(R.string.device_ip_info_first_paragraph))
                appendLine()
                appendLine()
                append(stringResource(R.string.device_ip_info_second_paragraph))
            },
        onDismiss = dropUnlessResumed { navigator.goBack() },
    )
}
