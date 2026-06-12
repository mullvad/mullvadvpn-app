package net.mullvad.mullvadvpn.feature.addtime.impl.verificationpending

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.VerificationPendingNavKey
import net.mullvad.mullvadvpn.feature.addtime.impl.R
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme

@Preview
@Composable
private fun PreviewVerificationPendingDialog() {
    AppTheme { VerificationPendingDialog(paymentStatus = PaymentStatus.PENDING, onClose = {}) }
}

@Composable
fun VerificationPending(navArgs: VerificationPendingNavKey, navigator: Navigator) {
    VerificationPendingDialog(
        paymentStatus = navArgs.paymentStatus,
        onClose = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun VerificationPendingDialog(paymentStatus: PaymentStatus, onClose: () -> Unit) {
    AlertDialog(
        icon = {}, // Makes it look a bit more balanced
        title = {
            Text(
                text =
                    stringResource(
                        id =
                            when (paymentStatus) {
                                PaymentStatus.PENDING -> R.string.verifying_purchase
                                PaymentStatus.VERIFICATION_IN_PROGRESS ->
                                    R.string.verifying_purchase_error
                            }
                    )
            )
        },
        text = {
            Text(
                text =
                    stringResource(
                        id =
                            when (paymentStatus) {
                                PaymentStatus.PENDING -> R.string.payment_pending_dialog_message
                                PaymentStatus.VERIFICATION_IN_PROGRESS ->
                                    R.string.verification_failed_dialog_message
                            }
                    ),
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
