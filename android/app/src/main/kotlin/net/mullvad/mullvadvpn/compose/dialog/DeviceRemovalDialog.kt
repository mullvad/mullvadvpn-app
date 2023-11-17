package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.model.Device

@Preview
@Composable
private fun PreviewShowDeviceRemovalDialog() {
    DeviceRemovalDialog(
        onDismiss = {},
        onConfirm = {},
        device = Device("test", "test", byteArrayOf(), "test")
    )
}

@Composable
fun DeviceRemovalDialog(onDismiss: () -> Unit, onConfirm: () -> Unit, device: Device) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.padding(top = 0.dp).fillMaxWidth()
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_alert),
                    contentDescription = "Remove",
                    modifier = Modifier.width(50.dp).height(50.dp)
                )
            }
        },
        text = {
            val htmlFormattedDialogText =
                textResource(id = R.string.max_devices_confirm_removal_description, device.displayName())

            HtmlText(htmlFormattedString = htmlFormattedDialogText, textSize = 16.sp.value)
        },
        dismissButton = {
            NegativeButton(
                onClick = onConfirm,
                text = stringResource(id = R.string.confirm_removal)
            )
        },
        confirmButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = onDismiss,
                text = stringResource(id = R.string.back)
            )
        },
        containerColor = MaterialTheme.colorScheme.background
    )
}
