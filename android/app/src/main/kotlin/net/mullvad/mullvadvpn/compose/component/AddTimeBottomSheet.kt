package net.mullvad.mullvadvpn.compose.component

import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Redeem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SheetState
import androidx.compose.material3.SheetValue
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.Dp.Companion.Hairline
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.preview.AddMoreTimeUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AddMoreTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.payment.ProductIds.OneMonth
import net.mullvad.mullvadvpn.lib.payment.ProductIds.ThreeMonths
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeSideEffect
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview(
    "Loading|oss|LoadingSitePayment|" +
        "PaymentLoading|NoPayment|NoProductsFound|PaymentAvailable|PaymentPending|PaymentError"
)
@Composable
private fun PreviewPaymentBottomSheet(
    @PreviewParameter(AddMoreTimeUiStatePreviewParameterProvider::class)
    state: Lc<Unit, AddMoreTimeUiState>
) {
    AppTheme {
        AddTimeBottomSheetContent(
            state = state,
            sheetState =
                SheetState(
                    skipPartiallyExpanded = true,
                    density = Density(1f),
                    initialValue = SheetValue.Expanded,
                ),
            onPurchaseBillingProductClick = {},
            onPlayPaymentInfoClick = {},
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            closeBottomSheet = {},
            onRetryFetchProducts = {},
            resetPurchaseState = {},
            onSuccessfulPurchase = {},
            retryPurchase = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddTimeBottomSheet(
    visible: Boolean,
    onRedeemVoucherClick: () -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onHideBottomSheet: () -> Unit,
) {
    val viewModel: AddMoreTimeViewModel = koinViewModel<AddMoreTimeViewModel>()
    val uiState by viewModel.uiState.collectAsStateWithLifecycle()

    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()
    val onCloseBottomSheet: (animate: Boolean) -> Unit = { animate ->
        if (animate) {
            scope.launch { sheetState.hide() }.invokeOnCompletion { onHideBottomSheet() }
        } else {
            onHideBottomSheet()
        }
    }

    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            is AddMoreTimeSideEffect.OpenAccountManagementPageInBrowser -> {
                openAccountPage(sideEffect.token)
                onCloseBottomSheet(true)
            }
        }
    }

    val activity = LocalActivity.current
    if (visible) {
        AddTimeBottomSheetContent(
            state = uiState,
            sheetState = sheetState,
            onPurchaseBillingProductClick = {
                viewModel.startBillingPayment(productId = it, activityProvider = { activity!! })
            },
            onPlayPaymentInfoClick = onPlayPaymentInfoClick,
            onSitePaymentClick = viewModel::onManageAccountClick,
            onRetryFetchProducts = viewModel::fetchPaymentAvailability,
            onRedeemVoucherClick = onRedeemVoucherClick,
            resetPurchaseState = { viewModel.onClosePurchaseResultDialog(false) },
            onSuccessfulPurchase = {
                viewModel.onClosePurchaseResultDialog(true)
                onCloseBottomSheet(true)
            },
            retryPurchase = { productId ->
                viewModel.startBillingPayment(
                    productId = productId,
                    activityProvider = { activity!! },
                )
            },
            closeBottomSheet = onCloseBottomSheet,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddTimeBottomSheetContent(
    state: Lc<Unit, AddMoreTimeUiState>,
    sheetState: SheetState,
    onPurchaseBillingProductClick: (ProductId) -> Unit = {},
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onRetryFetchProducts: () -> Unit,
    resetPurchaseState: () -> Unit,
    retryPurchase: (ProductId) -> Unit,
    onSuccessfulPurchase: () -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    val backgroundColor = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor = MaterialTheme.colorScheme.onSurface
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { closeBottomSheet(false) },
    ) {
        when (state) {
            is Lc.Loading ->
                Loading(backgroundColor = backgroundColor, onBackgroundColor = onBackgroundColor)
            is Lc.Content ->
                Content(
                    state = state.value,
                    backgroundColor = backgroundColor,
                    onBackgroundColor = onBackgroundColor,
                    onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                    onPlayPaymentInfoClick = onPlayPaymentInfoClick,
                    onSitePaymentClick = onSitePaymentClick,
                    onRedeemVoucherClick = onRedeemVoucherClick,
                    onRetryFetchProducts = onRetryFetchProducts,
                    closeBottomSheet = closeBottomSheet,
                    resetPurchaseState = resetPurchaseState,
                    onSuccessfulPurchase = onSuccessfulPurchase,
                    retryPurchase = retryPurchase,
                )
        }
    }
}

@Composable
private fun Content(
    state: AddMoreTimeUiState,
    backgroundColor: Color,
    onBackgroundColor: Color,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onRetryFetchProducts: () -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
    resetPurchaseState: () -> Unit,
    onSuccessfulPurchase: () -> Unit,
    retryPurchase: (ProductId) -> Unit,
) {
    if (state.purchaseState != null) {
        PurchaseState(
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            purchaseState = state.purchaseState,
            resetPurchaseState = resetPurchaseState,
            onSuccessfulPurchase = onSuccessfulPurchase,
            retryPurchase = retryPurchase,
        )
    } else {
        Products(
            billingPaymentState = state.billingPaymentState,
            showSitePayment = state.showSitePayment,
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onPurchaseBillingProductClick = onPurchaseBillingProductClick,
            onPlayPaymentInfoClick = onPlayPaymentInfoClick,
            onSitePaymentClick = onSitePaymentClick,
            onRedeemVoucherClick = onRedeemVoucherClick,
            onRetryFetchProducts = onRetryFetchProducts,
            closeBottomSheet = closeBottomSheet,
        )
    }
}

@Composable
private fun PurchaseState(
    backgroundColor: Color,
    onBackgroundColor: Color,
    purchaseState: PurchaseState,
    resetPurchaseState: () -> Unit,
    onSuccessfulPurchase: () -> Unit,
    retryPurchase: (ProductId) -> Unit,
) {
    when (purchaseState) {
        // Fetching products and obfuscated id loading state
        PurchaseState.Connecting -> {
            PurchaseStateLoading(title = stringResource(R.string.connecting))
        }
        PurchaseState.VerificationStarted -> {
            PurchaseStateLoading(title = stringResource(R.string.loading_verifying))
        }
        // Pending state
        PurchaseState.VerifyingPurchase -> {
            PurchaseStateVerification(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                resetPurchaseState = resetPurchaseState,
            )
        }
        // Success state
        is PurchaseState.Success -> {
            PurchaseStateSuccess(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                productId = purchaseState.productId,
                onSuccessfulPurchase = onSuccessfulPurchase,
            )
        }
        // Error states
        is PurchaseState.Error.TransactionIdError -> {
            PurchaseStateError(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                message = stringResource(R.string.payment_obfuscation_id_error_dialog_message),
                productId = purchaseState.productId,
                resetPurchaseState = resetPurchaseState,
                retryPurchase = retryPurchase,
            )
        }
        is PurchaseState.Error.OtherError -> {
            PurchaseStateError(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                message = stringResource(R.string.payment_billing_error_dialog_message),
                productId = purchaseState.productId,
                resetPurchaseState = resetPurchaseState,
                retryPurchase = retryPurchase,
            )
        }
    }
}

@Composable
private fun PurchaseStateVerification(
    onBackgroundColor: Color,
    backgroundColor: Color,
    resetPurchaseState: () -> Unit,
) {
    SheetTitle(
        title = stringResource(id = R.string.verifying_purchase),
        onBackgroundColor = onBackgroundColor,
        backgroundColor = backgroundColor,
    )
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            Modifier.fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenTopMargin),
    ) {
        Text(
            text = stringResource(id = R.string.payment_pending_dialog_message),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        PrimaryButton(
            text = stringResource(R.string.close),
            onClick = resetPurchaseState,
            modifier = Modifier.padding(top = Dimens.mediumPadding),
        )
    }
}

@Composable
private fun PurchaseStateLoading(title: String) {
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier = Modifier.fillMaxWidth().padding(all = Dimens.sideMargin),
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumPadding))
        MullvadLinearProgressIndicator()
    }
}

@Composable
private fun PurchaseStateSuccess(
    onBackgroundColor: Color,
    backgroundColor: Color,
    productId: ProductId,
    onSuccessfulPurchase: () -> Unit,
) {
    SheetTitle(
        title = stringResource(id = R.string.time_added),
        onBackgroundColor = onBackgroundColor,
        backgroundColor = backgroundColor,
    )
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            Modifier.fillMaxWidth()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenTopMargin),
    ) {
        Text(
            text =
                when (productId.value) {
                    OneMonth -> stringResource(R.string.days_were_added_30)
                    ThreeMonths -> stringResource(R.string.days_were_added_90)
                    else -> {
                        error("Unknown product: $productId")
                    }
                },
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
        PrimaryButton(
            text = stringResource(R.string.close),
            onClick = onSuccessfulPurchase,
            modifier = Modifier.padding(top = Dimens.mediumPadding),
        )
    }
}

@Composable
private fun PurchaseStateError(
    onBackgroundColor: Color,
    backgroundColor: Color,
    message: String,
    productId: ProductId,
    resetPurchaseState: () -> Unit,
    retryPurchase: (ProductId) -> Unit,
) {
    SheetTitle(
        title = stringResource(id = R.string.error_occurred),
        onBackgroundColor = onBackgroundColor,
        backgroundColor = backgroundColor,
    )
    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
    Text(
        text = message,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
    )
    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
    PrimaryButton(
        text = stringResource(R.string.retry),
        onClick = { retryPurchase(productId) },
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
    )
    Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
    PrimaryButton(
        text = stringResource(R.string.close),
        onClick = resetPurchaseState,
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
    )
}

@Composable
private fun Products(
    billingPaymentState: PaymentState?,
    showSitePayment: Boolean,
    backgroundColor: Color,
    onBackgroundColor: Color,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onRetryFetchProducts: () -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    SheetTitle(
        title = stringResource(id = R.string.add_time),
        onBackgroundColor = onBackgroundColor,
        backgroundColor = backgroundColor,
    )
    billingPaymentState?.let {
        PlayPayment(
            modifier = Modifier.fillMaxWidth(),
            billingPaymentState = billingPaymentState,
            onPurchaseBillingProductClick = onPurchaseBillingProductClick,
            onInfoClick = onPlayPaymentInfoClick,
            onRetryFetchProducts = onRetryFetchProducts,
        )
        HorizontalDivider(color = onBackgroundColor, thickness = Hairline)
    }
    if (showSitePayment) {
        IconCell(
            imageVector = Icons.AutoMirrored.Filled.OpenInNew,
            title = stringResource(id = R.string.buy_credit),
            onClick = { onSitePaymentClick() },
            titleColor = onBackgroundColor,
            background = backgroundColor,
        )
    }
    IconCell(
        imageVector = Icons.Default.Redeem,
        title = stringResource(id = R.string.redeem_voucher),
        titleColor = onBackgroundColor,
        onClick = {
            onRedeemVoucherClick()
            closeBottomSheet(true)
        },
        background = backgroundColor,
    )
}

@Composable
private fun ColumnScope.Loading(onBackgroundColor: Color, backgroundColor: Color) {
    SheetTitle(
        title = stringResource(id = R.string.add_time),
        onBackgroundColor = onBackgroundColor,
        backgroundColor = backgroundColor,
    )
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
}

@Composable
private fun SheetTitle(title: String, onBackgroundColor: Color, backgroundColor: Color) {
    HeaderCell(text = title, background = backgroundColor)
    HorizontalDivider(
        color = onBackgroundColor,
        modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
    )
    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
}
