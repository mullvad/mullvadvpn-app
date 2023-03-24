package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite

@Composable
fun ContentBlockersInfo(onDismiss: () -> Unit) {
    InfoDialog(
        info = stringResource(id = R.string.dns_content_blockers_info),
        warnings = stringResource(id = R.string.dns_content_blockers_warning),
        onDismiss
    )
}

@Composable
fun MalwareInfo(onDismiss: () -> Unit) {
    InfoDialog(info = null, warnings = stringResource(id = R.string.malware_info), onDismiss)
}

@Composable
fun InfoDialog(info: String?, warnings: String?, onDismiss: () -> Unit) {
    var verticalSpacing = 24.dp
    AlertDialog(
        onDismissRequest = { onDismiss() },
        title = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(44.dp),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = "",
                tint = MullvadWhite,
            )
        },
        text = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.padding(top = verticalSpacing),
            ) {
                if (info != null) {
                    Text(
                        text = info,
                        color = colorResource(id = R.color.white),
                        fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                        fontStyle = FontStyle.Normal,
                        textAlign = TextAlign.Start,
                        modifier = Modifier.padding(bottom = verticalSpacing).fillMaxWidth()
                    )
                }
                if (warnings != null) {
                    Text(
                        text = warnings,
                        color = colorResource(id = R.color.white),
                        fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                        fontStyle = FontStyle.Normal,
                        textAlign = TextAlign.Start,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        },
        buttons = {
            Button(
                modifier =
                Modifier.wrapContentHeight()
                    .padding(all = dimensionResource(id = R.dimen.medium_padding))
                    .defaultMinSize(minHeight = dimensionResource(id = R.dimen.button_height))
                    .fillMaxWidth(),
                colors =
                ButtonDefaults.buttonColors(
                    backgroundColor = colorResource(id = R.color.blue),
                    contentColor = colorResource(id = R.color.white),
                ),
                onClick = { onDismiss() },
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
        backgroundColor = colorResource(id = R.color.darkBlue),
    )
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTwoLongItems() {
    val longPreviewText =
        "This is a sample changelog item of a Compose Preview visualization. " +
            "The purpose of this specific sample text is to visualize a long text that will " +
            "result in multiple lines in the changelog dialog."

    InfoDialog(
        info = longPreviewText,
        warnings = longPreviewText,
        onDismiss = {},
    )
}
