package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun PreviewPaymentCompletedDialog() {
    PaymentCompletedDialog {}
}

@Composable
fun PaymentCompletedDialog(onClose: () -> Unit) {
    PaymentDialog(
        title = stringResource(id = R.string.payment_completed_dialog_title),
        message = stringResource(id = R.string.payment_completed_dialog_message),
        icon = R.drawable.icon_success,
        onConfirmClick = onClose,
        confirmText = stringResource(id = R.string.got_it),
        onDismissRequest = onClose
    )
}
