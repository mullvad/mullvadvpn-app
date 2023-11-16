package net.mullvad.mullvadvpn.compose.dialog.payment

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription

@Preview
@Composable
private fun PreviewVerificationPendingDialog() {
    AppTheme { VerificationPendingDialog(onClose = {}) }
}

@Composable
fun VerificationPendingDialog(onClose: () -> Unit) {
    AlertDialog(
        icon = {}, // Makes it look a bit more balanced
        title = {
            Text(
                text = stringResource(id = R.string.payment_pending_dialog_title),
                style = MaterialTheme.typography.headlineSmall
            )
        },
        text = {
            Text(
                text = stringResource(id = R.string.payment_pending_dialog_message),
                style = MaterialTheme.typography.bodySmall
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        textContentColor =
            MaterialTheme.colorScheme.onBackground
                .copy(alpha = AlphaDescription)
                .compositeOver(MaterialTheme.colorScheme.background),
        onDismissRequest = onClose,
        confirmButton = {
            PrimaryButton(text = stringResource(id = R.string.got_it), onClick = onClose)
        }
    )
}
