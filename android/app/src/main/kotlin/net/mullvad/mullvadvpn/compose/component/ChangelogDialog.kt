package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material.AlertDialog
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.DialogProperties
import net.mullvad.mullvadvpn.R

@Composable
fun ChangelogDialog(
    changesList: List<String>,
    version: String,
    onDismiss: () -> Unit
) {
    AlertDialog(
        onDismissRequest = {
            onDismiss()
        },
        title = {
            Text(
                text = version,
                color = colorResource(id = R.color.white),
                fontSize = 30.sp,
                fontStyle = FontStyle.Normal,
                textAlign = TextAlign.Center,
                modifier = Modifier
                    .fillMaxWidth()
            )
        },

        text = {
            Column {
                Text(
                    text = stringResource(R.string.changes_dialog_subtitle),
                    fontSize = 18.sp,
                    color = Color.White,
                    modifier = Modifier
                        .padding(
                            vertical = dimensionResource(id = R.dimen.medium_padding)
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
            Button(
                modifier = Modifier
                    .wrapContentHeight()
                    .padding(all = dimensionResource(id = R.dimen.medium_padding))
                    .defaultMinSize(
                        minHeight = dimensionResource(id = R.dimen.button_height)
                    )
                    .fillMaxWidth(),
                colors = ButtonDefaults.buttonColors(
                    backgroundColor = colorResource(id = R.color.blue),
                    contentColor = colorResource(id = R.color.white)
                ),
                onClick = {
                    onDismiss()
                }

            ) {
                Text(
                    text = stringResource(R.string.changes_dialog_dismiss_button),
                    fontSize = 18.sp
                )
            }
        },
        properties = DialogProperties(
            dismissOnClickOutside = true,
            dismissOnBackPress = true,
        ),
        backgroundColor = colorResource(id = R.color.darkBlue)
    )
}

@Preview
@Composable
private fun PreviewChangelogDialogWithSingleShortItem() {
    ChangelogDialog(
        changesList = listOf("Item 1"),
        version = "1111.1",
        onDismiss = {}
    )
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTwoLongItems() {
    val longPreviewText = "This is a sample changelog item of a Compose Preview visualization. " +
        "The purpose of this specific sample text is to visualize a long text that will result " +
        "in multiple lines in the changelog dialog."

    ChangelogDialog(
        changesList = listOf(longPreviewText, longPreviewText),
        version = "1111.1",
        onDismiss = {}
    )
}

@Preview
@Composable
private fun PreviewChangelogDialogWithTenShortItems() {
    ChangelogDialog(
        changesList = listOf(
            "Item 1",
            "Item 2",
            "Item 3",
            "Item 4",
            "Item 5",
            "Item 6",
            "Item 7",
            "Item 8",
            "Item 9",
            "Item 10"
        ),
        version = "1111.1",
        onDismiss = {}
    )
}
