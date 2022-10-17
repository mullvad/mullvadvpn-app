package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R

@Composable
fun ShowChangesDialog(
    changesList: List<String>,
    version: String,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = {
            onDismiss()
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
            Column {
                Text(
                    text = stringResource(R.string.changesHeader),
                    fontSize = 18.sp,
                    color = Color.White,
                    modifier = Modifier
                        .padding(
                            vertical = 16.dp
                        )
                )

                changesList.forEach { changeItem ->
                    ChangeListItem(
                        text = changeItem
                    )
                }
            }
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
                        onDismiss()
                    }
                ) {
                    Text(
                        text = stringResource(R.string.gotIt),
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
