package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.payment.model.PurchaseResult
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Composable
@Preview
private fun PreviewPurchaseResultDialogPurchaseStarted() {
    AppTheme {
        PurchaseResultDialog(
            purchaseResult = PurchaseResult.PurchaseStarted,
            retry = {},
            onCloseDialog = {}
        )
    }
}

@Composable
@Preview
private fun PreviewPurchaseResultDialogVerificationStarted() {
    AppTheme {
        PurchaseResultDialog(
            purchaseResult = PurchaseResult.VerificationStarted,
            retry = {},
            onCloseDialog = {}
        )
    }
}

@Composable
@Preview
private fun PreviewPurchaseResultDialogPurchaseCompleted() {
    AppTheme {
        PurchaseResultDialog(
            purchaseResult = PurchaseResult.PurchaseCompleted,
            retry = {},
            onCloseDialog = {}
        )
    }
}

@Composable
@Preview
private fun PreviewPurchaseResultTransactionIdError() {
    AppTheme {
        PurchaseResultDialog(
            purchaseResult = PurchaseResult.Error.TransactionIdError(null),
            retry = {},
            onCloseDialog = {}
        )
    }
}

@Composable
@Preview
private fun PreviewPurchaseResultVerificationError() {
    AppTheme {
        PurchaseResultDialog(
            purchaseResult = PurchaseResult.Error.VerificationError(null),
            retry = {},
            onCloseDialog = {}
        )
    }
}

@Composable
fun PurchaseResultDialog(
    purchaseResult: PurchaseResult,
    retry: () -> Unit,
    onCloseDialog: (success: Boolean) -> Unit
) {
    var showPurchaseDialog by remember { mutableStateOf<PurchaseDialog?>(null) }

    val onClose = { success: Boolean ->
        showPurchaseDialog = null
        onCloseDialog(success)
    }

    LaunchedEffect(key1 = purchaseResult) {
        showPurchaseDialog =
            when (purchaseResult) {
                // Idle states
                PurchaseResult.PurchaseCancelled,
                PurchaseResult.BillingFlowStarted,
                is PurchaseResult.Error.BillingError -> {
                    // Show nothing
                    null
                }
                // Loading states
                PurchaseResult.PurchaseStarted,
                PurchaseResult.VerificationStarted -> PurchaseDialog.Loading
                // Pending state
                PurchaseResult.PurchasePending -> PurchaseDialog.PurchasePending
                // Success state
                PurchaseResult.PurchaseCompleted -> PurchaseDialog.PurchaseCompleted
                // Error states
                is PurchaseResult.Error.TransactionIdError -> PurchaseDialog.GenericError
                is PurchaseResult.Error.VerificationError -> PurchaseDialog.VerificationError
            }
    }

    when (showPurchaseDialog) {
        // Loading states
        PurchaseDialog.Loading -> LoadingDialog(text = stringResource(id = R.string.connecting))
        // Pending state
        PurchaseDialog.PurchasePending -> PurchasePendingDialog(onClose = { onClose(false) })
        // Success state
        PurchaseDialog.PurchaseCompleted -> PurchaseCompletedDialog(onClose = { onClose(true) })
        // Error states
        is PurchaseDialog.GenericError ->
            PurchaseErrorDialog(
                title = stringResource(id = R.string.error_occurred),
                message = stringResource(id = R.string.try_again),
                onClose = { onClose(false) }
            )
        is PurchaseDialog.VerificationError ->
            PurchaseErrorDialog(
                title = stringResource(id = R.string.payment_verification_error_dialog_title),
                message = stringResource(id = R.string.payment_verification_error_dialog_message),
                retry = {
                    onClose(false)
                    retry()
                },
                onClose = { onClose(false) }
            )
        else -> {}
    }
}

@Composable
private fun PurchasePendingDialog(onClose: () -> Unit) {
    BasePaymentDialog(
        title = stringResource(id = R.string.payment_pending_dialog_title),
        message = stringResource(id = R.string.payment_pending_dialog_message),
        icon = R.drawable.icon_success,
        onConfirmClick = onClose,
        confirmText = stringResource(id = R.string.got_it),
        onDismissRequest = onClose
    )
}

@Composable
private fun PurchaseCompletedDialog(onClose: () -> Unit) {
    BasePaymentDialog(
        title = stringResource(id = R.string.payment_completed_dialog_title),
        message = stringResource(id = R.string.payment_completed_dialog_message),
        icon = R.drawable.icon_success,
        onConfirmClick = onClose,
        confirmText = stringResource(id = R.string.got_it),
        onDismissRequest = onClose
    )
}

@Composable
private fun PurchaseErrorDialog(
    title: String,
    message: String,
    retry: (() -> Unit)? = null,
    onClose: () -> Unit
) {
    BasePaymentDialog(
        title = title,
        message = message,
        icon = R.drawable.icon_fail,
        onConfirmClick = onClose,
        confirmText = stringResource(id = R.string.cancel),
        onDismissRequest = onClose,
        dismissText = retry?.let { stringResource(id = R.string.try_again) },
        onDismissClick = retry
    )
}

private sealed interface PurchaseDialog {
    data object Loading : PurchaseDialog

    data object PurchasePending : PurchaseDialog

    data object PurchaseCompleted : PurchaseDialog

    data object GenericError : PurchaseDialog

    data object VerificationError : PurchaseDialog
}
