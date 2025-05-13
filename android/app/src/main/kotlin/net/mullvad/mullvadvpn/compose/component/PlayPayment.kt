package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
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
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG

@Preview
@Composable
private fun PreviewPlayPaymentPaymentAvailable() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("one_month"),
                                    price = ProductPrice("$10"),
                                    status = null,
                                ),
                                PaymentProduct(
                                    productId = ProductId("three_months"),
                                    price = ProductPrice("$30"),
                                    status = null,
                                ),
                            )
                    ),
                onPurchaseBillingProductClick = {},
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenBottomMargin),
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentLoading() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            PlayPayment(
                billingPaymentState = PaymentState.Loading,
                onPurchaseBillingProductClick = {},
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenBottomMargin),
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentPaymentPending() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("one_month"),
                                    price = ProductPrice("$10"),
                                    status = PaymentStatus.PENDING,
                                ),
                                PaymentProduct(
                                    productId = ProductId("three_months"),
                                    price = ProductPrice("$30"),
                                    status = null,
                                ),
                            )
                    ),
                onPurchaseBillingProductClick = {},
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenBottomMargin),
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentVerificationInProgress() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            PlayPayment(
                billingPaymentState =
                    PaymentState.PaymentAvailable(
                        products =
                            listOf(
                                PaymentProduct(
                                    productId = ProductId("one_month"),
                                    price = ProductPrice("$10"),
                                    status = null,
                                ),
                                PaymentProduct(
                                    productId = ProductId("three_months"),
                                    price = ProductPrice("$30"),
                                    status = PaymentStatus.VERIFICATION_IN_PROGRESS,
                                ),
                            )
                    ),
                onPurchaseBillingProductClick = {},
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenBottomMargin),
            )
        }
    }
}

@Preview
@Composable
private fun PreviewPlayPaymentError() {
    AppTheme {
        Box(modifier = Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            PlayPayment(
                billingPaymentState = PaymentState.Error.Billing,
                onPurchaseBillingProductClick = {},
                onInfoClick = {},
                modifier = Modifier.padding(Dimens.screenBottomMargin),
            )
        }
    }
}

@Composable
fun PlayPayment(
    billingPaymentState: PaymentState,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onInfoClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    when (billingPaymentState) {
        PaymentState.Loading -> {
            Loading(modifier = modifier)
        }
        PaymentState.NoPayment,
        PaymentState.NoProductsFounds -> {
            // Show nothing
        }
        is PaymentState.PaymentAvailable -> {
            PaymentAvailable(
                modifier = modifier,
                billingPaymentState = billingPaymentState,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onInfoClick = onInfoClick,
            )
        }
        is PaymentState.Error -> {
            Error(modifier = modifier)
        }
    }
}

@Composable
private fun Loading(modifier: Modifier = Modifier) {
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .border(
                    width = Dimens.borderWidth,
                    color = MaterialTheme.colorScheme.onSurface,
                    shape = MaterialTheme.shapes.extraSmall,
                )
                .padding(all = Dimens.smallPadding)
    ) {
        MullvadCircularProgressIndicatorSmall()
    }
}

@Composable
private fun PaymentAvailable(
    billingPaymentState: PaymentState.PaymentAvailable,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onInfoClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier) {
        val statusMessage =
            when (billingPaymentState.products.status()) {
                PaymentStatus.PENDING -> stringResource(id = R.string.payment_status_pending)

                PaymentStatus.VERIFICATION_IN_PROGRESS ->
                    stringResource(id = R.string.verifying_purchase)

                else -> null
            }
        statusMessage?.let {
            Row(verticalAlignment = Alignment.Bottom) {
                Text(
                    style = MaterialTheme.typography.labelLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    text = statusMessage,
                    modifier = Modifier.padding(bottom = Dimens.smallPadding),
                )
                IconButton(
                    onClick = onInfoClick,
                    modifier = Modifier.testTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG),
                ) {
                    Icon(
                        imageVector = Icons.Default.Info,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
            }
        }
        Column(
            modifier =
                Modifier.border(
                        width = Dimens.borderWidth,
                        color = MaterialTheme.colorScheme.onSurface,
                        shape = MaterialTheme.shapes.extraSmall,
                    )
                    .padding(all = Dimens.smallPadding),
            verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing),
        ) {
            billingPaymentState.products.forEach { product ->
                VariantButton(
                    text =
                        when (product.productId.value) {
                            ProductIds.OneMonth ->
                                stringResource(
                                    id = R.string.add_30_days_time_x,
                                    product.price.value,
                                )

                            ProductIds.ThreeMonths ->
                                stringResource(
                                    id = R.string.add_90_days_time_x,
                                    product.price.value,
                                )

                            else -> {
                                // We have somehow requested a product that is not supported
                                error("ProductId ${product.productId.value} is not supported")
                            }
                        },
                    onClick = { onPurchaseBillingProductClick(product.productId) },
                    isEnabled = statusMessage == null,
                )
            }
        }
    }
}

@Composable
private fun Error(modifier: Modifier) {
    Column(
        modifier =
            modifier.border(
                width = Dimens.borderWidth,
                color = MaterialTheme.colorScheme.onSurface,
                shape = MaterialTheme.shapes.extraSmall,
            )
    ) {
        Text(
            text = stringResource(id = R.string.payment_billing_error_dialog_title),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(Dimens.smallPadding),
        )
    }
}

private fun List<PaymentProduct>.status(): PaymentStatus? {
    return this.firstOrNull { it.status != null }?.status
}
