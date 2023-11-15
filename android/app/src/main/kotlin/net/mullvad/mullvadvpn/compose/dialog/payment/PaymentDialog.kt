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
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription

@Preview
@Composable
private fun PreviewPaymentDialogPurchaseCompleted() {
    AppTheme {
        PaymentDialog(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_completed_dialog_title,
                    message = R.string.payment_completed_dialog_message,
                    icon = PaymentDialogIcon.SUCCESS,
                    confirmAction = PaymentDialogAction.Close,
                    successfulPayment = true
                ),
            retryPurchase = {},
            onCloseDialog = {}
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogPurchasePending() {
    AppTheme {
        PaymentDialog(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_pending_dialog_title,
                    message = R.string.payment_pending_dialog_message,
                    confirmAction = PaymentDialogAction.Close,
                    closeOnDismiss = true
                ),
            retryPurchase = {},
            onCloseDialog = {}
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogGenericError() {
    AppTheme {
        PaymentDialog(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.error_occurred,
                    message = R.string.try_again,
                    icon = PaymentDialogIcon.FAIL,
                    confirmAction = PaymentDialogAction.Close
                ),
            retryPurchase = {},
            onCloseDialog = {}
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogLoading() {
    AppTheme {
        PaymentDialog(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.loading_connecting,
                    icon = PaymentDialogIcon.LOADING,
                    closeOnDismiss = false
                ),
            retryPurchase = {},
            onCloseDialog = {}
        )
    }
}

@Preview
@Composable
private fun PreviewPaymentDialogPaymentAvailabilityError() {
    AppTheme {
        PaymentDialog(
            paymentDialogData =
                PaymentDialogData(
                    title = R.string.payment_billing_error_dialog_title,
                    message = R.string.payment_billing_error_dialog_message,
                    icon = PaymentDialogIcon.FAIL,
                    confirmAction = PaymentDialogAction.Close,
                    dismissAction = PaymentDialogAction.RetryPurchase(productId = ProductId("test"))
                ),
            retryPurchase = {},
            onCloseDialog = {}
        )
    }
}

@Composable
fun PaymentDialog(
    paymentDialogData: PaymentDialogData,
    retryPurchase: (ProductId) -> Unit,
    onCloseDialog: (isPaymentSuccessful: Boolean) -> Unit
) {
    val clickResolver: (action: PaymentDialogAction) -> Unit = {
        when (it) {
            is PaymentDialogAction.RetryPurchase -> retryPurchase(it.productId)
            is PaymentDialogAction.Close -> onCloseDialog(paymentDialogData.successfulPayment)
        }
    }
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
                onCloseDialog(paymentDialogData.successfulPayment)
            }
        },
        dismissButton = {
            paymentDialogData.dismissAction?.let {
                PrimaryButton(
                    text = stringResource(id = it.message),
                    onClick = { clickResolver(it) }
                )
            }
        },
        confirmButton = {
            paymentDialogData.confirmAction?.let {
                PrimaryButton(
                    text = stringResource(id = it.message),
                    onClick = { clickResolver(it) }
                )
            }
        }
    )
}
