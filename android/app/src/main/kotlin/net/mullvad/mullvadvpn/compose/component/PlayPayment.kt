package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.ProductIds
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
                onInfoClick = {},
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
                onInfoClick = {},
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
                onInfoClick = {},
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
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenVerticalMargin)
            )
        }
    }
}

@Composable
fun PlayPayment(
    billingPaymentState: PaymentState,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onInfoClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    when (billingPaymentState) {
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
                        Row(verticalAlignment = Alignment.Bottom) {
                            Text(
                                style = MaterialTheme.typography.labelLarge,
                                color = MaterialTheme.colorScheme.onBackground,
                                text = statusMessage,
                                modifier = Modifier.padding(bottom = Dimens.smallPadding)
                            )
                            IconButton(onClick = onInfoClick) {
                                Icon(
                                    painter = painterResource(id = R.drawable.icon_info),
                                    contentDescription = null,
                                    tint = MaterialTheme.colorScheme.onBackground
                                )
                            }
                        }
                    }
                    VariantButton(
                        text =
                            stringResource(id = R.string.add_30_days_time_x, product.price.value),
                        onClick = { onPurchaseBillingProductClick(product.productId) },
                        isEnabled = product.status == null
                    )
                }
            }
        }
        // Show the button without the price
        is PaymentState.Error -> {
            Column(modifier = modifier) {
                VariantButton(
                    text = stringResource(id = R.string.add_30_days_time),
                    onClick = { onPurchaseBillingProductClick(ProductId(ProductIds.OneMonth)) }
                )
            }
        }
    }
}
