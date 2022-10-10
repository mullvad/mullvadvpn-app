package net.mullvad.mullvadvpn.compose.component

import android.content.Context
import androidx.compose.foundation.layout.*
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.ui.serviceconnection.appVersionInfoCache
import net.mullvad.mullvadvpn.util.toBulletList
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel

@Composable
fun ShowChangesDialog(
    context: Context,
    changesViewModel: AppChangesViewModel,
    serviceConnectionManager: ServiceConnectionManager,
    onBackPressed: () -> Unit
) {
    var version: String? = serviceConnectionManager.appVersionInfoCache()?.version
    if (version.isNullOrEmpty()) version = BuildConfig.VERSION_NAME
    var changesHeader = "<h4>${context.getString(R.string.changesHeader)}</h4>\n"
    AlertDialog(
        onDismissRequest = {
            changesViewModel.setDialogShowed()
        },
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier
                    .padding(top = 0.dp)
                    .fillMaxWidth()
            ) {
                Text(
                    text = version,
                    color = colorResource(id = R.color.white),
                    fontSize = 30.sp,
                    fontStyle = FontStyle.Normal
                )
            }
        },

        text = {
            HtmlText(
                htmlFormattedString = changesHeader +
                        changesViewModel.getChangesList().toBulletList(),
                textSize = 14.sp.value
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
                        contentColor = Color.White
                    ),
                    onClick = {
                        onBackPressed()
                    }
                ) {
                    Text(
                        text = context.getString(R.string.gotIt),
                        fontSize = 18.sp
                    )
                }

            }
        },
        properties = DialogProperties(
            dismissOnClickOutside = true,
            dismissOnBackPress = true,
        ),
        backgroundColor = colorResource(id = R.color.darkBlue)
    )
}
