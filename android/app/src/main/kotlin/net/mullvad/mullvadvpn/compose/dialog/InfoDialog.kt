package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.lib.theme.MullvadWhite

@Preview
@Composable
private fun PreviewChangelogDialogWithTwoLongItems() {
    val longPreviewText =
        "This is a sample changelog item of a Compose Preview visualization. " +
            "The purpose of this specific sample text is to visualize a long text that will " +
            "result in multiple lines in the changelog dialog."

    InfoDialog(message = longPreviewText, additionalInfo = longPreviewText, onDismiss = {})
}

@Composable
fun InfoDialog(message: String, additionalInfo: String? = null, onDismiss: () -> Unit) {
    val verticalSpacing = 24.dp
    val iconHeight = 44.dp
    AlertDialog(
        onDismissRequest = { onDismiss() },
        title = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(iconHeight),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = "",
                tint = MullvadWhite
            )
        },
        text = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text = message,
                    color = colorResource(id = R.color.white),
                    fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                    fontStyle = FontStyle.Normal,
                    textAlign = TextAlign.Start,
                    modifier = Modifier.fillMaxWidth()
                )
                if (additionalInfo != null) {
                    Spacer(modifier = Modifier.height(verticalSpacing))
                    HtmlText(
                        htmlFormattedString = additionalInfo,
                        textColor = colorResource(id = R.color.white).toArgb(),
                        textSize = dimensionResource(id = R.dimen.text_small).value,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        },
        confirmButton = {
            Button(
                modifier = Modifier.wrapContentHeight().fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.blue),
                        contentColor = colorResource(id = R.color.white)
                    ),
                onClick = { onDismiss() },
                shape = MaterialTheme.shapes.small
            ) {
                Text(
                    text = stringResource(R.string.changes_dialog_dismiss_button),
                    fontSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp,
                )
            }
        },
        properties =
            DialogProperties(
                dismissOnClickOutside = true,
                dismissOnBackPress = true,
            ),
        containerColor = colorResource(id = R.color.darkBlue)
    )
}
