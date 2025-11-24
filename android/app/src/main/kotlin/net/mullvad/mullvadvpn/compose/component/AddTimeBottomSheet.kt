package net.mullvad.mullvadvpn.compose.component

import androidx.activity.compose.LocalActivity
import androidx.compose.animation.AnimatedContent
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Redeem
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Sell
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.SmallPrimaryButton
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.preview.AddMoreTimeUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.AddTimeUiState
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.PurchaseState
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.payment.ProductIds.OneMonth
import net.mullvad.mullvadvpn.lib.payment.ProductIds.ThreeMonths
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.tag.ADD_TIME_BOTTOM_SHEET_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.AddMoreTimeSideEffect
import net.mullvad.mullvadvpn.viewmodel.AddTimeViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview(
    "Loading|oss|LoadingSitePayment|" +
        "PaymentLoading|NoPayment|NoProductsFound|PaymentAvailable|PaymentPending|PaymentError"
)
@Composable
private fun PreviewPaymentBottomSheet(
    @PreviewParameter(AddMoreTimeUiStatePreviewParameterProvider::class)
    state: Lc<Unit, AddTimeUiState>
) {
    AppTheme {
        AddTimeBottomSheetContent(
            state = state,
            sheetState =
                SheetState(
                    skipPartiallyExpanded = true,
                    positionalThreshold = { 0f },
                    velocityThreshold = { 0f },
                    initialValue = SheetValue.Expanded,
                ),
            onPurchaseBillingProductClick = {},
            onPlayPaymentInfoClick = {},
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            closeBottomSheet = {},
            onRetryFetchProducts = {},
            resetPurchaseState = {},
            closeSheetAndResetPurchaseState = {},
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
    val viewModel: AddTimeViewModel = koinViewModel<AddTimeViewModel>()
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
            resetPurchaseState = { viewModel.resetPurchaseResult() },
            closeSheetAndResetPurchaseState = {
                viewModel.resetPurchaseResult()
                onCloseBottomSheet(true)
            },
            closeBottomSheet = onCloseBottomSheet,
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddTimeBottomSheetContent(
    state: Lc<Unit, AddTimeUiState>,
    sheetState: SheetState,
    onPurchaseBillingProductClick: (ProductId) -> Unit = {},
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onRetryFetchProducts: () -> Unit,
    resetPurchaseState: () -> Unit,
    closeSheetAndResetPurchaseState: (Boolean) -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
) {
    val backgroundColor = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor = MaterialTheme.colorScheme.onSurface
    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = {
            resetPurchaseState()
            closeBottomSheet(false)
        },
    ) {
        when (state) {
            is Lc.Loading ->
                Loading(backgroundColor = backgroundColor, onBackgroundColor = onBackgroundColor)
            is Lc.Content ->
                Content(
                    state = state.value,
                    internetBlocked = state.value.tunnelStateBlocked,
                    backgroundColor = backgroundColor,
                    onBackgroundColor = onBackgroundColor,
                    onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                    onPlayPaymentInfoClick = onPlayPaymentInfoClick,
                    onSitePaymentClick = onSitePaymentClick,
                    onRedeemVoucherClick = onRedeemVoucherClick,
                    onRetryFetchProducts = onRetryFetchProducts,
                    closeBottomSheet = closeBottomSheet,
                    resetPurchaseState = resetPurchaseState,
                    closeSheetAndResetPurchaseState = closeSheetAndResetPurchaseState,
                )
        }
    }
}

@Composable
private fun Content(
    state: AddTimeUiState,
    internetBlocked: Boolean,
    backgroundColor: Color,
    onBackgroundColor: Color,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onRetryFetchProducts: () -> Unit,
    closeBottomSheet: (animate: Boolean) -> Unit,
    resetPurchaseState: () -> Unit,
    closeSheetAndResetPurchaseState: (Boolean) -> Unit,
) {
    AnimatedContent(targetState = state) { state ->
        Column {
            if (state.purchaseState != null) {
                PurchaseState(
                    backgroundColor = backgroundColor,
                    onBackgroundColor = onBackgroundColor,
                    purchaseState = state.purchaseState,
                    resetPurchaseState = resetPurchaseState,
                    closeSheetAndResetPurchaseState = closeSheetAndResetPurchaseState,
                )
            } else {
                Products(
                    billingPaymentState = state.billingPaymentState,
                    showSitePayment = state.showSitePayment,
                    internetBlocked = internetBlocked,
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
    }
}

@Composable
private fun ColumnScope.PurchaseState(
    backgroundColor: Color,
    onBackgroundColor: Color,
    purchaseState: PurchaseState,
    resetPurchaseState: () -> Unit,
    closeSheetAndResetPurchaseState: (Boolean) -> Unit,
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
                closeSheet = closeSheetAndResetPurchaseState,
            )
        }
        // Success state
        is PurchaseState.Success -> {
            PurchaseStateSuccess(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                productId = purchaseState.productId,
                onSuccessfulPurchase = closeSheetAndResetPurchaseState,
            )
        }
        // Error states
        is PurchaseState.Error.TransactionIdError -> {
            PurchaseStateError(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                title = stringResource(R.string.payment_obfuscation_id_error_dialog_title),
                message = stringResource(R.string.payment_obfuscation_id_error_dialog_message),
                resetPurchaseState = resetPurchaseState,
            )
        }
        is PurchaseState.Error.OtherError -> {
            PurchaseStateError(
                backgroundColor = backgroundColor,
                onBackgroundColor = onBackgroundColor,
                title = stringResource(R.string.payment_billing_error_dialog_title),
                message = stringResource(R.string.payment_billing_error_dialog_message),
                resetPurchaseState = resetPurchaseState,
            )
        }
    }
}

@Composable
private fun PurchaseStateVerification(
    onBackgroundColor: Color,
    backgroundColor: Color,
    closeSheet: (Boolean) -> Unit,
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
        SmallPrimaryButton(
            text = stringResource(R.string.close),
            onClick = { closeSheet(false) },
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
    onSuccessfulPurchase: (Boolean) -> Unit,
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
        SmallPrimaryButton(
            text = stringResource(R.string.close),
            onClick = { onSuccessfulPurchase(true) },
            modifier = Modifier.padding(top = Dimens.mediumPadding),
        )
    }
}

@Composable
private fun ColumnScope.PurchaseStateError(
    onBackgroundColor: Color,
    backgroundColor: Color,
    title: String,
    message: String,
    resetPurchaseState: () -> Unit,
) {
    SheetTitle(
        title = title,
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
    SmallPrimaryButton(
        text = stringResource(android.R.string.ok),
        onClick = resetPurchaseState,
        modifier = Modifier.padding(top = Dimens.mediumPadding).align(Alignment.CenterHorizontally),
    )
}

@Composable
private fun Products(
    billingPaymentState: PaymentState?,
    internetBlocked: Boolean,
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
            onBackgroundColor = onBackgroundColor,
            onPurchaseBillingProductClick = onPurchaseBillingProductClick,
            onInfoClick = onPlayPaymentInfoClick,
            onRetryFetchProducts = onRetryFetchProducts,
        )
    }
    if (showSitePayment) {
        if (internetBlocked) {
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
                    text = stringResource(R.string.app_is_blocking_internet),
                    modifier = Modifier.padding(start = Dimens.miniPadding),
                )
            }
        }
        IconCell(
            imageVector = Icons.Outlined.Sell,
            title = stringResource(id = R.string.buy_credit),
            titleColor =
                onBackgroundColor.copy(
                    alpha = if (internetBlocked) AlphaDisabled else AlphaVisible
                ),
            onClick = { onSitePaymentClick() },
            enabled = !internetBlocked,
            endIcon = {
                Icon(
                    imageVector = Icons.AutoMirrored.Filled.OpenInNew,
                    tint =
                        onBackgroundColor.copy(
                            alpha = if (internetBlocked) AlphaDisabled else AlphaVisible
                        ),
                    contentDescription = null,
                )
            },
        )
        HorizontalDivider(
            modifier = Modifier.height(Dimens.thinBorderWidth),
            color = onBackgroundColor,
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
    HeaderCell(
        text = title,
        background = backgroundColor,
        modifier = Modifier.testTag(ADD_TIME_BOTTOM_SHEET_TITLE_TEST_TAG),
    )
    HorizontalDivider(
        color = onBackgroundColor,
        modifier = Modifier.padding(horizontal = Dimens.mediumPadding),
    )
    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
}
