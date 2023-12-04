package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun DeviceNameInfoDialog(navigator: DestinationsNavigator) {
    InfoDialog(
        message =
            buildString {
                appendLine(stringResource(id = R.string.device_name_info_first_paragraph))
                appendLine()
                appendLine(stringResource(id = R.string.device_name_info_second_paragraph))
                appendLine()
                append(stringResource(id = R.string.device_name_info_third_paragraph))
            },
        onDismiss = navigator::navigateUp
    )
}
