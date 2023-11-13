package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewPlayPaymentPaymentAvailable() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("test"),
                                    price = ProductPrice("$10"),
                                    status = null
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
private fun PreviewPlayPaymentLoading() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPayment(
                billingPaymentState = PaymentState.Loading,
                onPurchaseBillingProductClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentPaymentPending() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("test"),
                                    price = ProductPrice("$10"),
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
private fun PreviewPlayPaymentVerificationInProgress() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("test"),
                                    price = ProductPrice("$10"),
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
fun PlayPayment(
    billingPaymentState: PaymentState,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    modifier: Modifier = Modifier
) {
    when (billingPaymentState) {
        PaymentState.Error.Billing,
        PaymentState.Error.Generic -> {
            // We show some kind of dialog error at the top
        }
        PaymentState.Loading -> {
            Column(modifier = modifier.fillMaxWidth()) {
                MullvadCircularProgressIndicatorSmall(modifier = modifier)
            }
        }
        PaymentState.NoPayment,
        PaymentState.NoProductsFounds -> {
            // Show nothing
        }
        is PaymentState.PaymentAvailable -> {
            billingPaymentState.products.forEach { product ->
                Column(modifier = modifier) {
                    val statusMessage =
                        when (product.status) {
                            PaymentStatus.PENDING ->
                                stringResource(id = R.string.payment_status_pending)
                            PaymentStatus.VERIFICATION_IN_PROGRESS ->
                                stringResource(
                                    id = R.string.payment_status_verification_in_progress
                                )
                            else -> null
                        }
                    statusMessage?.let {
                        Text(
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onBackground,
                            text = statusMessage,
                            modifier = Modifier.padding(bottom = Dimens.smallPadding)
                        )
                    }
                    VariantButton(
                        text = stringResource(id = R.string.add_30_days_time_x, product.price),
                        onClick = { onPurchaseBillingProductClick(product.productId) },
                        isEnabled = product.status == null
                    )
                }
            }
        }
    }
}
