package net.mullvad.mullvadvpn.compose.dialog.payment

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.coroutines.flow.collect
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.viewmodel.PaymentUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.PaymentViewModel
import org.koin.androidx.compose.koinViewModel

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

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun Payment(productId: ProductId, resultBackNavigator: ResultBackNavigator<Boolean>) {
    val vm = koinViewModel<PaymentViewModel>()
    val uiState by vm.uiState.collectAsState()

    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect {
            when (it) {
                is PaymentUiSideEffect.PaymentCancelled ->
                    resultBackNavigator.navigateBack(result = false)
            }
        }
    }

    val context = LocalContext.current
    LaunchedEffect(Unit) { vm.startBillingPayment(productId) { context.getActivity()!! } }
    PaymentDialog(
        paymentDialogData = uiState.paymentDialogData,
        retryPurchase = { vm.startBillingPayment(it) { context.getActivity()!! } },
        onCloseDialog = { resultBackNavigator.navigateBack(result = it) }
    )
}

@Composable
fun PaymentDialog(
    paymentDialogData: PaymentDialogData,
    retryPurchase: (ProductId) -> Unit = {},
    onCloseDialog: (isPaymentSuccessful: Boolean) -> Unit = {}
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

private fun Context.getActivity(): Activity? {
    return when (this) {
        is Activity -> this
        is ContextWrapper -> this.baseContext.getActivity()
        else -> null
    }
}
