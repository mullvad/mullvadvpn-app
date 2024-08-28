package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
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
        DeleteConfirmationDialog(message = "Do you want to delete Cookie?", errorMessage = null)
    }
}

@Preview
@Composable
private fun PreviewDeleteConfirmationDialogError() {
    AppTheme {
        DeleteConfirmationDialog(
            message = "Do you want to delete Cookie?",
            errorMessage = "An error occured",
        )
    }
}

@Composable
fun DeleteConfirmationDialog(
    message: String,
    errorMessage: String?,
    onDelete: () -> Unit = {},
    onBack: () -> Unit = {},
) {
    AlertDialog(
        onDismissRequest = onBack,
        icon = {
            Icon(
                modifier = Modifier.fillMaxWidth().height(Dimens.dialogIconHeight),
                painter = painterResource(id = R.drawable.icon_alert),
                contentDescription = stringResource(id = R.string.remove_button),
                tint = MaterialTheme.colorScheme.error,
            )
        },
        title = {
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text(text = message)
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
                text = stringResource(id = R.string.cancel),
            )
        },
        confirmButton = {
            NegativeButton(onClick = onDelete, text = stringResource(id = R.string.delete))
        },
        containerColor = MaterialTheme.colorScheme.surface,
    )
}
