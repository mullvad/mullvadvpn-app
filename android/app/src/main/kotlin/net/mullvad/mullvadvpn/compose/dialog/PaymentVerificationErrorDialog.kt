package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun PreviewPaymentVerificationErrorDialog() {
    PaymentVerificationErrorDialog(onTryAgain = {}, onClose = {})
}

@Composable
fun PaymentVerificationErrorDialog(onTryAgain: () -> Unit, onClose: () -> Unit) {
    PaymentDialog(
        title = stringResource(id = R.string.payment_verification_error_dialog_title),
        message = stringResource(id = R.string.payment_verification_error_dialog_message),
        icon = R.drawable.icon_fail,
        onConfirmClick = onClose,
        confirmText = stringResource(id = R.string.cancel),
        onDismissRequest = onClose,
        dismissText = stringResource(id = R.string.try_again),
        onDismissClick = onTryAgain
    )
}
