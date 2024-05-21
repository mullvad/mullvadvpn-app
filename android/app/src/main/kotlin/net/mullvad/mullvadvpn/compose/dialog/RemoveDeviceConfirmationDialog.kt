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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.EmptyResultBackNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.util.generateDevice
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.model.DeviceId

@Preview
@Composable
private fun PreviewRemoveDeviceConfirmationDialog() {
    AppTheme {
        RemoveDeviceConfirmationDialog(EmptyResultBackNavigator(), device = generateDevice())
    }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun RemoveDeviceConfirmationDialog(navigator: ResultBackNavigator<DeviceId>, device: Device) {
    AlertDialog(
        onDismissRequest = navigator::navigateBack,
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = stringResource(id = R.string.remove_button),
                tint = Color.Unspecified
            )
        },
        text = {
            val htmlFormattedDialogText =
                textResource(
                    id = R.string.max_devices_confirm_removal_description,
                    device.displayName()
                )

            HtmlText(htmlFormattedString = htmlFormattedDialogText, textSize = 16.sp.value)
        },
        dismissButton = {
            NegativeButton(
                onClick = { navigator.navigateBack(result = device.id) },
                text = stringResource(id = R.string.confirm_removal)
            )
        },
        confirmButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = { navigator.navigateBack() },
                text = stringResource(id = R.string.back)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
