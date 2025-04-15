package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialog
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.compose.preview.DevicePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.util.appendTextWithStyledSubstring

@Preview
@Composable
private fun PreviewManageDevicesRemoveConfirmationDialog(
    @PreviewParameter(DevicePreviewParameterProvider::class) device: Device
) {
    AppTheme { ManageDevicesRemoveConfirmationDialog(EmptyResultBackNavigator(), device = device) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ManageDevicesRemoveConfirmationDialog(
    navigator: ResultBackNavigator<DeviceId>,
    device: Device,
) {
    InfoConfirmationDialog(
        navigator = navigator,
        confirmValue = device.id,
        titleType = InfoConfirmationDialogTitleType.IconOnly,
        confirmButtonTitle = textResource(R.string.remove_button),
        cancelButtonTitle = textResource(R.string.cancel),
    ) {
        Column {
            Text(
                text = getText(device),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                style = MaterialTheme.typography.labelMedium,
            )
        }
    }
}

@Composable
private fun getText(device: Device): AnnotatedString {
    val line1 =
        textResource(
            id = R.string.manage_devices_confirm_removal_description_line1,
            device.displayName(),
        )

    val line2 = textResource(id = R.string.manage_devices_confirm_removal_description_line2)

    return buildAnnotatedString {
        appendTextWithStyledSubstring(
            text = line1,
            substring = device.displayName(),
            substringStyle = SpanStyle(color = Color.White),
        )
        append("\n")
        append(line2)
    }
}
