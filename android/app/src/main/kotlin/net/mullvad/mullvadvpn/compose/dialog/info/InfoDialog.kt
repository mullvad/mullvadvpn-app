package net.mullvad.mullvadvpn.compose.dialog.info

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.DialogProperties
import androidx.core.text.HtmlCompat
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar

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
fun InfoDialog(
    title: String? = null,
    message: String,
    additionalInfo: CharSequence? = null,
    showIcon: Boolean = true,
    onDismiss: () -> Unit,
    confirmButton: @Composable () -> Unit = {
        PrimaryButton(
            modifier = Modifier.wrapContentHeight().fillMaxWidth(),
            text = stringResource(R.string.got_it),
            onClick = onDismiss,
        )
    },
    dismissButton: @Composable (() -> Unit)? = null,
) {
    InfoDialog(
        title = title,
        message = AnnotatedString(message),
        additionalInfo = additionalInfo,
        showIcon = showIcon,
        onDismiss = onDismiss,
        confirmButton = confirmButton,
        dismissButton = dismissButton,
    )
}

@Composable
fun InfoDialog(
    title: String? = null,
    message: AnnotatedString,
    additionalInfo: CharSequence? = null,
    showIcon: Boolean = true,
    onDismiss: () -> Unit,
    confirmButton: @Composable () -> Unit = {
        PrimaryButton(
            modifier = Modifier.wrapContentHeight().fillMaxWidth(),
            text = stringResource(R.string.got_it),
            onClick = onDismiss,
        )
    },
    dismissButton: @Composable (() -> Unit)? = null,
) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
        icon =
            if (showIcon) {
                {
                    Icon(
                        modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                        imageVector = Icons.Default.Info,
                        contentDescription = "",
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
            } else null,
        title =
            if (title != null) {
                @Composable { Text(title) }
            } else {
                null
            },
        text = {
            val scrollState = rememberScrollState()
            Column(
                Modifier.drawVerticalScrollbar(
                        scrollState,
                        MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(scrollState),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                Text(
                    text = message,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelLarge,
                    modifier = Modifier.fillMaxWidth(),
                )
                if (additionalInfo != null) {
                    Spacer(modifier = Modifier.height(Dimens.verticalSpace))
                    val annotated: AnnotatedString =
                        when (additionalInfo) {
                            is AnnotatedString -> additionalInfo
                            is String -> {
                                val htmlAnnotated =
                                    HtmlCompat.fromHtml(
                                            additionalInfo,
                                            HtmlCompat.FROM_HTML_MODE_COMPACT,
                                        )
                                        .toAnnotatedString(FontWeight.Bold)
                                // fromHtml may add a trailing newline when using HTML tags, so we
                                // remove it
                                AnnotatedString(
                                    htmlAnnotated.substring(0, htmlAnnotated.trimEnd().length)
                                )
                            }
                            else ->
                                error("Unsupported additionalInfo type ${additionalInfo::class}")
                        }
                    Text(
                        text = annotated,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        style = MaterialTheme.typography.labelLarge,
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }
        },
        confirmButton = confirmButton,
        dismissButton = dismissButton,
        properties = DialogProperties(dismissOnClickOutside = true, dismissOnBackPress = true),
        containerColor = MaterialTheme.colorScheme.surface,
    )
}
