package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun DeviceNameInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message =
        buildString {
            appendLine(stringResource(id = R.string.device_name_info_part1))
            appendLine()
            appendLine(stringResource(id = R.string.device_name_info_part2))
            appendLine()
            appendLine(stringResource(id = R.string.device_name_info_part3))
        },
        onDismiss = onDismiss
    )
}
