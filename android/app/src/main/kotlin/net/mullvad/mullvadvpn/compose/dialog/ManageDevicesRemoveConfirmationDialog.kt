package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialog
import net.mullvad.mullvadvpn.compose.dialog.info.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.compose.preview.DevicePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewManageDevicesRemoveConfirmationDialog(
    @PreviewParameter(DevicePreviewParameterProvider::class) device: Device
) {
    AppTheme { ManageDevicesRemoveConfirmation(EmptyResultBackNavigator(), device = device) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun ManageDevicesRemoveConfirmation(navigator: ResultBackNavigator<DeviceId>, device: Device) {
    InfoConfirmationDialog(
        navigator = navigator,
        confirmValue = device.id,
        titleType = InfoConfirmationDialogTitleType.IconAndTitle(title = device.titleText()),
        confirmButtonTitle = stringResource(R.string.remove_button),
        cancelButtonTitle = stringResource(R.string.cancel),
    ) {
        Text(
            text = stringResource(id = R.string.manage_devices_confirm_removal_description_line2),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelLarge,
        )
    }
}

@Composable
private fun Device.titleText(): String =
    stringResource(id = R.string.manage_devices_confirm_removal_description_line1, displayName())
