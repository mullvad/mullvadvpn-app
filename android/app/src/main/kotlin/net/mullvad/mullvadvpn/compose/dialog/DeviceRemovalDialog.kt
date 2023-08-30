package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.lib.common.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.model.Device

@Composable
fun ShowDeviceRemovalDialog(onDismiss: () -> Unit, onConfirm: () -> Unit, device: Device) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
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
                textResource(
                    id = R.string.max_devices_confirm_removal_description,
                    device.name.capitalizeFirstCharOfEachWord()
                )

            HtmlText(htmlFormattedString = htmlFormattedDialogText, textSize = 16.sp.value)
        },
        dismissButton = {
            Button(
                modifier =
                    Modifier.height(dimensionResource(id = R.dimen.button_height))
                        .defaultMinSize(
                            minWidth = 0.dp,
                            minHeight = dimensionResource(id = R.dimen.button_height)
                        )
                        .fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.red),
                        contentColor = Color.White
                    ),
                onClick = onConfirm,
                shape = MaterialTheme.shapes.small
            ) {
                Text(text = stringResource(id = R.string.confirm_removal), fontSize = 18.sp)
            }
        },
        confirmButton = {
            Button(
                contentPadding = PaddingValues(0.dp),
                modifier =
                    Modifier.focusRequester(FocusRequester())
                        .height(dimensionResource(id = R.dimen.button_height))
                        .defaultMinSize(
                            minWidth = 0.dp,
                            minHeight = dimensionResource(id = R.dimen.button_height)
                        )
                        .fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.blue),
                        contentColor = Color.White
                    ),
                onClick = { onDismiss() },
                shape = MaterialTheme.shapes.small
            ) {
                Text(text = stringResource(id = R.string.back), fontSize = 18.sp)
            }
        },
        containerColor = colorResource(id = R.color.darkBlue)
    )
}
