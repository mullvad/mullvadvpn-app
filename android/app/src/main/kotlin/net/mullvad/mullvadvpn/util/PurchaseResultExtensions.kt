package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentClickAction
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogAction
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogData
import net.mullvad.mullvadvpn.compose.dialog.payment.PaymentDialogIcon
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult

fun PurchaseResult.toPaymentDialogData(): PaymentDialogData? =
    when (this) {
        // Idle states
        PurchaseResult.Completed.Cancelled,
        PurchaseResult.BillingFlowStarted,
        is PurchaseResult.Error.BillingError -> {
            // Show nothing
            null
        }
        // Fetching products and obfuscated id loading state
        PurchaseResult.FetchingProducts,
        PurchaseResult.FetchingObfuscationId ->
            PaymentDialogData(
                title = R.string.loading_connecting,
                icon = PaymentDialogIcon.LOADING,
                closeOnDismiss = false
            )
        // Verifying loading states
        PurchaseResult.VerificationStarted ->
            PaymentDialogData(
                title = R.string.loading_verifying,
                icon = PaymentDialogIcon.LOADING,
                closeOnDismiss = false
            )
        // Pending state
        PurchaseResult.Completed.Pending ->
            PaymentDialogData(
                title = R.string.payment_pending_dialog_title,
                message = R.string.payment_pending_dialog_message,
                confirmAction =
                    PaymentDialogAction(
                        message = R.string.got_it,
                        onClick = PaymentClickAction.CLOSE
                    )
            )
        // Success state
        PurchaseResult.Completed.Success ->
            PaymentDialogData(
                title = R.string.payment_completed_dialog_title,
                message = R.string.payment_completed_dialog_message,
                icon = PaymentDialogIcon.SUCCESS,
                confirmAction =
                    PaymentDialogAction(
                        message = R.string.got_it,
                        onClick = PaymentClickAction.CLOSE,
                    ),
                successfulPayment = true
            )
        // Error states
        is PurchaseResult.Error.TransactionIdError ->
            PaymentDialogData(
                title = R.string.payment_obfuscation_id_error_dialog_title,
                message = R.string.payment_obfuscation_id_error_dialog_message,
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
                    ),
                productId = this.productId
            )
        is PurchaseResult.Error.VerificationError ->
            PaymentDialogData(
                title = R.string.payment_verification_error_dialog_title,
                message = R.string.payment_verification_error_dialog_message,
                icon = PaymentDialogIcon.FAIL,
                confirmAction =
                    PaymentDialogAction(
                        message = R.string.cancel,
                        onClick = PaymentClickAction.CLOSE
                    ),
                dismissAction =
                    PaymentDialogAction(
                        message = R.string.try_again,
                        onClick = PaymentClickAction.RETRY_VERIFICATION
                    )
            )
        is PurchaseResult.Error.FetchProductsError,
        is PurchaseResult.Error.NoProductFound -> {
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
                    ),
                productId =
                    when (this) {
                        is PurchaseResult.Error.FetchProductsError -> this.productId
                        is PurchaseResult.Error.NoProductFound -> this.productId
                        else -> ProductId("")
                    }
            )
        }
    }
