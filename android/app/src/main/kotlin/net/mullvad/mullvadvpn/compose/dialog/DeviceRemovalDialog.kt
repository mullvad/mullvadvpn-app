package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusOrder
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
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel

@Composable
fun ShowDeviceRemovalDialog(viewModel: DeviceListViewModel, device: Device) {
    AlertDialog(
        onDismissRequest = {
            viewModel.clearStagedDevice()
        },
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .padding(top = 0.dp)
                    .fillMaxWidth()
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_alert),
                    contentDescription = "Remove",
                    modifier = Modifier
                        .width(50.dp)
                        .height(50.dp)
                )
            }
        },
        text = {
            val htmlFormattedDialogText = textResource(
                id = R.string.max_devices_confirm_removal_description,
                device.name.capitalizeFirstCharOfEachWord()
            ).let { introText ->
                if (device.ports.isNotEmpty()) {
                    introText.plus(" " + stringResource(id = R.string.port_removal_notice))
                } else {
                    introText
                }
            }

            HtmlText(
                htmlFormattedString = htmlFormattedDialogText,
                textSize = 16.sp.value
            )
        },
        buttons = {
            Column(
                Modifier
                    .padding(start = 16.dp, end = 16.dp, bottom = 16.dp)
            ) {
                Button(
                    modifier = Modifier
                        .height(dimensionResource(id = R.dimen.button_height))
                        .defaultMinSize(
                            minWidth = 0.dp,
                            minHeight = dimensionResource(id = R.dimen.button_height)
                        )
                        .fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        backgroundColor = colorResource(id = R.color.red),
                        contentColor = Color.White
                    ),
                    onClick = {
                        viewModel.confirmRemovalOfStagedDevice()
                    }
                ) {
                    Text(
                        text = stringResource(id = R.string.confirm_removal),
                        fontSize = 18.sp
                    )
                }
                Button(
                    contentPadding = PaddingValues(0.dp),
                    modifier = Modifier
                        .focusOrder(FocusRequester())
                        .padding(top = 16.dp)
                        .height(dimensionResource(id = R.dimen.button_height))
                        .defaultMinSize(
                            minWidth = 0.dp,
                            minHeight = dimensionResource(id = R.dimen.button_height)
                        )
                        .fillMaxWidth(),
                    colors = ButtonDefaults.buttonColors(
                        backgroundColor = colorResource(id = R.color.blue),
                        contentColor = Color.White
                    ),
                    onClick = {
                        viewModel.clearStagedDevice()
                    }
                ) {
                    Text(
                        text = stringResource(id = R.string.back),
                        fontSize = 18.sp
                    )
                }
            }
        },
        backgroundColor = colorResource(id = R.color.darkBlue)
    )
}
