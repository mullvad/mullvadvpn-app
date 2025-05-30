package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.outlined.Sell
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.preview.PlayPaymentPaymentStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG

@Preview(
    "Loading|NoPayment|NoProductsFound|Error.Generic|Error.Billing" +
        "|PaymentAvailable|PaymentAvailable.Pending|PaymentAvailable.VerificationInProgress"
)
@Composable
private fun PreviewPlayPayment(
    @PreviewParameter(PlayPaymentPaymentStatePreviewParameterProvider::class) state: PaymentState
) {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surfaceContainer)) {
            PlayPayment(
                billingPaymentState = state,
                onPurchaseBillingProductClick = {},
                onRetryFetchProducts = {},
                onInfoClick = {},
            )
        }
    }
}

@Composable
fun PlayPayment(
    billingPaymentState: PaymentState,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onRetryFetchProducts: () -> Unit,
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
            Error(modifier = modifier, retryFetchProducts = onRetryFetchProducts)
        }
    }
}

@Composable
private fun Loading(modifier: Modifier = Modifier) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            modifier
                .fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenTopMargin),
    ) {
        Text(
            text = stringResource(id = R.string.loading_products),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
        MullvadLinearProgressIndicator()
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

                null -> null
            }
        val enabled = statusMessage == null
        statusMessage?.let {
            Row(
                verticalAlignment = Alignment.Bottom,
                modifier =
                    Modifier.padding(start = Dimens.cellStartPadding, end = Dimens.cellStartPadding),
            ) {
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
        Column {
            billingPaymentState.products.forEach { product ->
                IconCell(
                    background = MaterialTheme.colorScheme.surfaceContainer,
                    titleColor =
                        if (enabled) {
                            MaterialTheme.colorScheme.onSurface
                        } else {
                            MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaDisabled)
                        },
                    imageVector = Icons.Outlined.Sell,
                    title =
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
                    enabled = enabled,
                )
            }
        }
    }
}

@Composable
private fun Error(modifier: Modifier, retryFetchProducts: () -> Unit) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            modifier.padding(vertical = Dimens.screenTopMargin, horizontal = Dimens.sideMargin),
    ) {
        Text(
            text = stringResource(id = R.string.failed_to_load_products),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
        )
        PrimaryButton(text = stringResource(R.string.retry), onClick = retryFetchProducts)
    }
}

private fun List<PaymentProduct>.status(): PaymentStatus? {
    return this.firstOrNull { it.status != null }?.status
}
