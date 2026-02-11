package net.mullvad.mullvadvpn.feature.addtime.impl

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Sell
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.lib.payment.ProductIds
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.ui.component.listitem.IconListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadLinearProgressIndicator
import net.mullvad.mullvadvpn.lib.ui.designsystem.SmallPrimaryButton
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

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
                onBackgroundColor = MaterialTheme.colorScheme.onSurface,
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
    onBackgroundColor: Color,
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
            return
        }
        is PaymentState.PaymentAvailable -> {
            PaymentAvailable(
                modifier = modifier,
                billingPaymentState = billingPaymentState,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onInfoClick = onInfoClick,
            )
        }
        is PaymentState.Error.Generic -> {
            Error(
                modifier = modifier,
                message = stringResource(id = R.string.failed_to_load_products),
                retryFetchProducts = onRetryFetchProducts,
            )
        }
        is PaymentState.Error.Billing -> {
            Error(
                modifier = modifier,
                message = stringResource(id = R.string.in_app_products_unavailable),
                retryFetchProducts = onRetryFetchProducts,
            )
        }
    }
    HorizontalDivider(color = onBackgroundColor, thickness = Dimens.thinBorderWidth)
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
    val statusMessage = billingPaymentState.products.status()?.message()
    Column(
        modifier =
            modifier
                .clickable(enabled = statusMessage != null, onClick = onInfoClick)
                .testTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG)
    ) {
        val enabled = statusMessage == null
        statusMessage?.let {
            Row(
                verticalAlignment = Alignment.CenterVertically,
                modifier =
                    Modifier.padding(start = Dimens.cellStartPadding, end = Dimens.cellStartPadding),
            ) {
                Icon(
                    modifier = Modifier.size(Dimens.smallIconSize),
                    imageVector = Icons.Outlined.Info,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurface,
                )
                Text(
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    text = statusMessage,
                    modifier = Modifier.padding(start = Dimens.miniPadding),
                )
            }
        }
        Column {
            billingPaymentState.products.forEach { product ->
                IconListItem(
                    leadingIcon = Icons.Outlined.Sell,
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
                    colors =
                        ListItemDefaults.colors(
                            containerColorParent = MaterialTheme.colorScheme.surfaceContainer,
                            headlineColor = MaterialTheme.colorScheme.onSurface,
                        ),
                    // Setting this to null if not enabled to fix a failing test due to performClick
                    // not respecting enabled for clickable
                    onClick =
                        if (enabled) {
                            { onPurchaseBillingProductClick(product.productId) }
                        } else null,
                    isEnabled = enabled,
                )
            }
        }
    }
}

@Composable
private fun Error(modifier: Modifier, message: String, retryFetchProducts: () -> Unit) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            modifier.padding(vertical = Dimens.screenTopMargin, horizontal = Dimens.sideMargin),
    ) {
        Text(
            text = message,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
            textAlign = TextAlign.Center,
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
        )
        SmallPrimaryButton(text = stringResource(R.string.retry), onClick = retryFetchProducts)
    }
}

private fun List<PaymentProduct>.status(): PaymentStatus? {
    return this.firstOrNull { it.status != null }?.status
}

@Composable
private fun PaymentStatus.message(): String =
    when (this) {
        PaymentStatus.PENDING -> stringResource(id = R.string.payment_status_pending_long)

        PaymentStatus.VERIFICATION_IN_PROGRESS -> stringResource(id = R.string.verifying_purchase)
    }
