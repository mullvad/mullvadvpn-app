package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewDeleteConfirmationDialog() {
    AppTheme {
        NegativeConfirmationDialog(
            message = "Do you want to delete Cookie?",
            errorMessage = null,
            onConfirm = {},
            onBack = {},
        )
    }
}

@Preview
@Composable
private fun PreviewDeleteConfirmationDialogError() {
    AppTheme {
        NegativeConfirmationDialog(
            message = "Do you want to delete Cookie?",
            errorMessage = "An error occured",
            onConfirm = {},
            onBack = {},
        )
    }
}

@Composable
fun NegativeConfirmationDialog(
    message: String,
    messageStyle: TextStyle? = null,
    errorMessage: String? = null,
    confirmationText: String = stringResource(id = R.string.delete),
    cancelText: String = stringResource(id = R.string.cancel),
    onConfirm: () -> Unit,
    onBack: () -> Unit,
) {
    NegativeConfirmationDialog(
        message = AnnotatedString(message),
        messageStyle = messageStyle,
        errorMessage = errorMessage,
        confirmationText = confirmationText,
        cancelText = cancelText,
        onConfirm = onConfirm,
        onBack = onBack,
    )
}

@Composable
fun NegativeConfirmationDialog(
    message: AnnotatedString,
    messageStyle: TextStyle? = null,
    errorMessage: String? = null,
    confirmationText: String = stringResource(id = R.string.delete),
    cancelText: String = stringResource(id = R.string.cancel),
    onConfirm: () -> Unit,
    onBack: () -> Unit,
) {
    AlertDialog(
        onDismissRequest = onBack,
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                imageVector = Icons.Default.Error,
                contentDescription = stringResource(id = R.string.remove_button),
                tint = MaterialTheme.colorScheme.error,
            )
        },
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(text = message, style = messageStyle ?: LocalTextStyle.current)
                if (errorMessage != null) {
                    Text(
                        text = errorMessage,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.error,
                        modifier = Modifier.padding(top = Dimens.smallPadding),
                    )
                }
            }
        },
        dismissButton = {
            PrimaryButton(
                modifier = Modifier.focusRequester(FocusRequester()),
                onClick = onBack,
                text = cancelText,
            )
        },
        confirmButton = { NegativeButton(onClick = onConfirm, text = confirmationText) },
        containerColor = MaterialTheme.colorScheme.surface,
    )
}
