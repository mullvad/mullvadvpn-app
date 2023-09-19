package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewPlayPaymentButtonPaymentAvailable() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPaymentButton(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = "test",
                                    price = "$10",
                                    status = PaymentStatus.AVAILABLE
                                )
                            )
                    ),
                onPurchaseBillingProductClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentButtonLoading() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPaymentButton(
                billingPaymentState = PaymentState.Loading,
                onPurchaseBillingProductClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentButtonPaymentPending() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPaymentButton(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = "test",
                                    price = "$10",
                                    status = PaymentStatus.PENDING
                                )
                            )
                    ),
                onPurchaseBillingProductClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentButtonVerificationInProgress() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPaymentButton(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = "test",
                                    price = "$10",
                                    status = PaymentStatus.VERIFICATION_IN_PROGRESS
                                )
                            )
                    ),
                onPurchaseBillingProductClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Composable
fun PlayPaymentButton(
    billingPaymentState: PaymentState,
    onPurchaseBillingProductClick: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    when (billingPaymentState) {
        PaymentState.Error.BillingError,
        PaymentState.Error.GenericError -> {
            // We show some kind of dialog error at the top
        }
        PaymentState.Loading -> {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.onBackground,
                modifier =
                    modifier.size(
                        width = Dimens.progressIndicatorSize,
                        height = Dimens.progressIndicatorSize
                    )
            )
        }
        PaymentState.NoPayment -> {
            // Show nothing
        }
        is PaymentState.PaymentAvailable -> {
            billingPaymentState.products.forEach { product ->
                Column(modifier = modifier) {
                    if (product.status != PaymentStatus.AVAILABLE) {
                        Text(
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onBackground,
                            text =
                                stringResource(
                                    id =
                                        if (
                                            product.status == PaymentStatus.VERIFICATION_IN_PROGRESS
                                        ) {
                                            R.string.payment_status_verification_in_progress
                                        } else {
                                            R.string.payment_status_pending
                                        }
                                ),
                            modifier = Modifier.padding(bottom = Dimens.smallPadding)
                        )
                    }
                    VariantButton(
                        text = stringResource(id = R.string.add_30_days_time_x, product.price),
                        onClick = { onPurchaseBillingProductClick(product.productId) },
                        isEnabled = product.status == PaymentStatus.AVAILABLE
                    )
                }
            }
        }
    }
}
