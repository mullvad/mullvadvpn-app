package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.toAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar

@Preview
@Composable
private fun PreviewChangelogDialogWithTwoLongItems() {
    val longPreviewText =
        "This is a sample changelog item of a Compose Preview visualization. " +
            "The purpose of this specific sample text is to visualize a long text that will " +
            "result in multiple lines in the changelog dialog."

    AppTheme {
        InfoDialog(message = longPreviewText, additionalInfo = longPreviewText, onDismiss = {})
    }
}

@Composable
fun InfoDialog(message: String, additionalInfo: String? = null, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = "",
                tint = MaterialTheme.colorScheme.onBackground
            )
        },
        text = {
            val scrollState = rememberScrollState()
            Column(
                Modifier.drawVerticalScrollbar(
                        scrollState,
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                    )
                    .verticalScroll(scrollState),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text = message,
                    color = MaterialTheme.colorScheme.onBackground,
                    style = MaterialTheme.typography.bodySmall,
                    modifier = Modifier.fillMaxWidth()
                )
                if (additionalInfo != null) {
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    val htmlFormattedString =
                        HtmlCompat.fromHtml(additionalInfo, HtmlCompat.FROM_HTML_MODE_COMPACT)
                    val annotated = htmlFormattedString.toAnnotatedString(FontWeight.Bold)
                    // fromHtml may add a trailing newline when using HTML tags, so we remove it
                    val trimmed = annotated.substring(0, annotated.trimEnd().length)
                    Text(
                        text = trimmed,
                        color = MaterialTheme.colorScheme.onBackground,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.fillMaxWidth()
                    )
                }
            }
        },
        confirmButton = {
            PrimaryButton(
                modifier = Modifier.wrapContentHeight().fillMaxWidth(),
                text = stringResource(R.string.got_it),
                onClick = onDismiss,
            )
        },
        properties =
            DialogProperties(
                dismissOnClickOutside = true,
                dismissOnBackPress = true,
            ),
        containerColor = MaterialTheme.colorScheme.background
    )
}
