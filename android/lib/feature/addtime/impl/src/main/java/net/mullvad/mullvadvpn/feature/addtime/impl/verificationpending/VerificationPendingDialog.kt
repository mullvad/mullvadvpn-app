package net.mullvad.mullvadvpn.feature.addtime.impl.verificationpending

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.impl.R
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewVerificationPendingDialog() {
    AppTheme { VerificationPendingDialog(onClose = {}) }
}

@Composable
fun VerificationPending(navigator: Navigator) {
    VerificationPendingDialog(onClose = dropUnlessResumed { navigator.goBack() })
}

@Composable
fun VerificationPendingDialog(onClose: () -> Unit) {
    AlertDialog(
        icon = {}, // Makes it look a bit more balanced
        title = { Text(text = stringResource(id = R.string.verifying_purchase)) },
        text = {
            Text(
                text = stringResource(id = R.string.payment_pending_dialog_message),
                style = MaterialTheme.typography.labelLarge,
            )
        },
        containerColor = MaterialTheme.colorScheme.surface,
        titleContentColor = MaterialTheme.colorScheme.onSurface,
        textContentColor = MaterialTheme.colorScheme.onSurfaceVariant,
        onDismissRequest = onClose,
        confirmButton = {
            PrimaryButton(text = stringResource(id = R.string.got_it), onClick = onClose)
        },
    )
}
