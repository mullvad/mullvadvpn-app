package net.mullvad.mullvadvpn.compose.dialog.payment

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription

@Preview
@Composable
private fun PreviewPaymentDialogPurchaseCompleted() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_completed_dialog_title,
                    message = R.string.payment_completed_dialog_message,
                    icon = PaymentDialogIcon.SUCCESS,
                    confirmAction =
                        PaymentDialogAction(message = R.string.got_it, PaymentClickAction.CLOSE),
                    successfulPayment = true
                ),
            onClick = { _, _ -> }
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogPurchasePending() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_pending_dialog_title,
                    message = R.string.payment_pending_dialog_message,
                    confirmAction =
                        PaymentDialogAction(message = R.string.got_it, PaymentClickAction.CLOSE),
                    closeOnDismiss = true
                ),
            onClick = { _, _ -> }
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogVerificationFailed() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_verification_error_dialog_title,
                    message = R.string.payment_verification_error_dialog_message,
                    icon = PaymentDialogIcon.FAIL,
                    confirmAction =
                        PaymentDialogAction(message = R.string.cancel, PaymentClickAction.CLOSE),
                    dismissAction =
                        PaymentDialogAction(
                            message = R.string.try_again,
                            PaymentClickAction.RETRY_VERIFICATION
                        ),
                    closeOnDismiss = true
                ),
            onClick = { _, _ -> }
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogGenericError() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.error_occurred,
                    message = R.string.try_again,
                    icon = PaymentDialogIcon.FAIL,
                    confirmAction =
                        PaymentDialogAction(
                            message = R.string.cancel,
                            onClick = PaymentClickAction.CLOSE
                        )
                ),
            onClick = { _, _ -> }
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogLoading() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.loading_connecting,
                    icon = PaymentDialogIcon.LOADING,
                    closeOnDismiss = false
                ),
            onClick = { _, _ -> }
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogPaymentAvailabilityError() {
    AppTheme {
        PaymentDialogContent(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_billing_error_dialog_title,
                    message = R.string.payment_billing_error_dialog_message,
                    icon = PaymentDialogIcon.FAIL,
                    confirmAction =
                        PaymentDialogAction(
                            message = R.string.cancel,
                            onClick = PaymentClickAction.CLOSE
                        ),
                    dismissAction =
                        PaymentDialogAction(
                            message = R.string.try_again,
                            onClick = PaymentClickAction.RETRY_PURCHASE
                        )
                ),
            onClick = { _, _ -> }
        )
    }
}

@Composable
fun PaymentDialog(
    paymentDialogData: PaymentDialogData,
    retryPurchase: (ProductId) -> Unit,
    retryVerification: () -> Unit,
    onCloseDialog: (isPaymentSuccessful: Boolean) -> Unit
) {
    PaymentDialogContent(
        paymentDialogData = paymentDialogData,
        onClick = { isPaymentSuccessful, clickAction ->
            when (clickAction) {
                PaymentClickAction.RETRY_PURCHASE -> retryPurchase(paymentDialogData.productId)
                PaymentClickAction.RETRY_VERIFICATION -> retryVerification()
                PaymentClickAction.CLOSE -> onCloseDialog(isPaymentSuccessful)
            }
        }
    )
}

@Composable
private fun PaymentDialogContent(
    paymentDialogData: PaymentDialogData,
    onClick: (isPaymentSuccessful: Boolean, clickAction: PaymentClickAction) -> Unit
) {
    AlertDialog(
        icon = {
            when (paymentDialogData.icon) {
                PaymentDialogIcon.SUCCESS ->
                    Icon(
                        painter = painterResource(id = R.drawable.icon_success),
                        contentDescription = null
                    )
                PaymentDialogIcon.FAIL ->
                    Icon(
                        painter = painterResource(id = R.drawable.icon_fail),
                        contentDescription = null
                    )
                PaymentDialogIcon.LOADING -> MullvadCircularProgressIndicatorMedium()
                else -> {}
            }
        },
        title = {
            paymentDialogData.title?.let {
                Text(
                    text = stringResource(id = paymentDialogData.title),
                    style = MaterialTheme.typography.headlineSmall
                )
            }
        },
        text =
            paymentDialogData.message?.let {
                {
                    Text(
                        text = stringResource(id = paymentDialogData.message),
                        style = MaterialTheme.typography.bodySmall
                    )
                }
            },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        iconContentColor = Color.Unspecified,
        textContentColor =
            MaterialTheme.colorScheme.onBackground
                .copy(alpha = AlphaDescription)
                .compositeOver(MaterialTheme.colorScheme.background),
        onDismissRequest = {
            if (paymentDialogData.closeOnDismiss) {
                onClick(paymentDialogData.successfulPayment, PaymentClickAction.CLOSE)
            }
        },
        dismissButton = {
            paymentDialogData.dismissAction?.let {
                NegativeButton(
                    text = stringResource(id = it.message),
                    onClick = { onClick(paymentDialogData.successfulPayment, it.onClick) }
                )
            }
        },
        confirmButton = {
            paymentDialogData.confirmAction?.let {
                PrimaryButton(
                    text = stringResource(id = it.message),
                    onClick = { onClick(paymentDialogData.successfulPayment, it.onClick) }
                )
            }
        }
    )
}
