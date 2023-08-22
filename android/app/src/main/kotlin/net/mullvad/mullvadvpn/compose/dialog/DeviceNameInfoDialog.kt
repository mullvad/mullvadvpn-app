package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun DeviceNameInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message = stringResource(id = R.string.local_network_sharing_info),
        additionalInfo =
            buildString {
                appendLine(stringResource(id = R.string.device_name_info_part1))
                appendLine(stringResource(id = R.string.device_name_info_part2))
                appendLine(stringResource(id = R.string.device_name_info_part3))
            },
        onDismiss = onDismiss
    )
}
