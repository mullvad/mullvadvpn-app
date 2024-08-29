package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.preview.DevicePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog(
    @PreviewParameter(DevicePreviewParameterProvider::class) device: Device
) {
    AppTheme { RemoveDeviceConfirmation(EmptyResultBackNavigator(), device = device) }
}

@Destination<RootGraph>(style = DestinationStyle.Dialog::class)
@Composable
fun RemoveDeviceConfirmation(navigator: ResultBackNavigator<DeviceId>, device: Device) {
    AlertDialog(
        onDismissRequest = dropUnlessResumed { navigator.navigateBack() },
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = stringResource(id = R.string.remove_button),
                tint = MaterialTheme.colorScheme.error,
            )
        },
        text = {
            val htmlFormattedDialogText =
                textResource(
                    id = R.string.max_devices_confirm_removal_description,
                    device.displayName(),
                )

            HtmlText(htmlFormattedString = htmlFormattedDialogText, textSize = 16.sp.value)
        },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = dropUnlessResumed { navigator.navigateBack() },
                text = stringResource(id = R.string.back),
            )
        },
        confirmButton = {
            NegativeButton(
                onClick = dropUnlessResumed { navigator.navigateBack(result = device.id) },
                text = stringResource(id = R.string.confirm_removal),
            )
        },
        containerColor = MaterialTheme.colorScheme.surface,
    )
}
