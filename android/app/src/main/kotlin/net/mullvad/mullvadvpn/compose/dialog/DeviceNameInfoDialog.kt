package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.mullvad.mullvadvpn.R

@Composable
fun DeviceNameInfoDialog(onDismiss: () -> Unit) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.device_name_info_first_paragraph))
                appendLine()
                appendLine(stringResource(id = R.string.device_name_info_second_paragraph))
                appendLine()
                append(stringResource(id = R.string.device_name_info_third_paragraph))
            },
        onDismiss = onDismiss
    )
}
