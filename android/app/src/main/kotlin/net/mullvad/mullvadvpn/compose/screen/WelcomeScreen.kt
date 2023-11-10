package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PlayPaymentButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.DeviceNameInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.PaymentAvailabilityErrorDialog
import net.mullvad.mullvadvpn.compose.dialog.PurchaseResultDialog
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.color.MullvadWhite
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel

@Preview
@Composable
private fun PreviewWelcomeScreen() {
    AppTheme {
        WelcomeScreen(
            showSitePayment = true,
            uiState =
                WelcomeUiState(
                    accountNumber = "4444555566667777",
                    deviceName = "Happy Mole",
                    billingPaymentState =
                        PaymentState.PaymentAvailable(
                            products = listOf(PaymentProduct("product", "$44", null))
                        )
                ),
            uiSideEffect = MutableSharedFlow<WelcomeViewModel.UiSideEffect>().asSharedFlow(),
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            onSettingsClick = {},
            onAccountClick = {},
            openConnectScreen = {},
            onPurchaseBillingProductClick = {},
            onRetryFetchProducts = {},
            onRetryVerification = {},
            onClosePurchaseResultDialog = {}
        )
    }
}

@Composable
fun WelcomeScreen(
    showSitePayment: Boolean,
    uiState: WelcomeUiState,
    uiSideEffect: SharedFlow<WelcomeViewModel.UiSideEffect>,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    openConnectScreen: () -> Unit,
    onPurchaseBillingProductClick: (productId: String) -> Unit,
    onRetryVerification: () -> Unit,
    onClosePurchaseResultDialog: (success: Boolean) -> Unit,
    onRetryFetchProducts: () -> Unit
) {
    val context = LocalContext.current
    LaunchedEffect(Unit) {
        uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                is WelcomeViewModel.UiSideEffect.OpenAccountView ->
                    context.openAccountPageInBrowser(uiSideEffect.token)
                WelcomeViewModel.UiSideEffect.OpenConnectScreen -> openConnectScreen()
            }
        }
    }

    uiState.purchaseResult?.let {
        PurchaseResultDialog(
            purchaseResult = uiState.purchaseResult,
            retry = onRetryVerification,
            onCloseDialog = onClosePurchaseResultDialog
        )
    }

    PaymentAvailabilityErrorDialog(
        paymentAvailability = uiState.billingPaymentState,
        retry = onRetryFetchProducts,
    )

    val scrollState = rememberScrollState()
    val snackbarHostState = remember { SnackbarHostState() }

    ScaffoldWithTopBar(
        topBarColor =
            if (uiState.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.inversePrimary
            } else {
                MaterialTheme.colorScheme.error
            },
        statusBarColor =
            if (uiState.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.inversePrimary
            } else {
                MaterialTheme.colorScheme.error
            },
        navigationBarColor = MaterialTheme.colorScheme.background,
        iconTintColor =
            if (uiState.tunnelState.isSecured()) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onError
                }
                .copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        snackbarHostState = snackbarHostState
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaScrollbar)
                    )
                    .background(color = MaterialTheme.colorScheme.primary)
        ) {
            // Welcome info area
            WelcomeInfo(snackbarHostState, uiState, showSitePayment)

            Spacer(modifier = Modifier.weight(1f))

            // Payment button area
            PaymentPanel(
                showSitePayment = showSitePayment,
                billingPaymentState = uiState.billingPaymentState,
                onSitePaymentClick = onSitePaymentClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick
            )
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    uiState: WelcomeUiState,
    showSitePayment: Boolean
) {
    Column {
        Text(
            text = stringResource(id = R.string.congrats),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        top = Dimens.screenVerticalMargin,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin
                    ),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis
        )
        Text(
            text = stringResource(id = R.string.here_is_your_account_number),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        horizontal = Dimens.sideMargin,
                        vertical = Dimens.smallPadding,
                    ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onPrimary
        )

        AccountNumberRow(snackbarHostState, uiState)

        DeviceNameRow(deviceName = uiState.deviceName)

        Text(
            text =
                buildString {
                    append(stringResource(id = R.string.pay_to_start_using))
                    if (showSitePayment) {
                        append(" ")
                        append(stringResource(id = R.string.add_time_to_account))
                    }
                },
            modifier =
                Modifier.padding(
                    top = Dimens.smallPadding,
                    bottom = Dimens.verticalSpace,
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin
                ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onPrimary
        )
    }
}

@Composable
private fun AccountNumberRow(snackbarHostState: SnackbarHostState, uiState: WelcomeUiState) {
    val copiedAccountNumberMessage = stringResource(id = R.string.copied_mullvad_account_number)
    val copyToClipboard = createCopyToClipboardHandle(snackbarHostState = snackbarHostState)
    val onCopyToClipboard = {
        copyToClipboard(uiState.accountNumber ?: "", copiedAccountNumberMessage)
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier =
            Modifier.fillMaxWidth()
                .clickable(onClick = onCopyToClipboard)
                .padding(horizontal = Dimens.sideMargin)
    ) {
        Text(
            text = uiState.accountNumber?.groupWithSpaces() ?: "",
            modifier = Modifier.weight(1f).padding(vertical = Dimens.smallPadding),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onPrimary
        )

        CopyAnimatedIconButton(onCopyToClipboard)
    }
}

@Composable
fun DeviceNameRow(deviceName: String?) {
    Row(
        modifier = Modifier.padding(horizontal = Dimens.sideMargin),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(
            modifier = Modifier.weight(1f, fill = false),
            text =
                buildString {
                    append(stringResource(id = R.string.device_name))
                    append(": ")
                    append(deviceName)
                },
            style = MaterialTheme.typography.bodySmall,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            color = MaterialTheme.colorScheme.onPrimary
        )

        var showDeviceNameDialog by remember { mutableStateOf(false) }
        IconButton(
            modifier = Modifier.align(Alignment.CenterVertically),
            onClick = { showDeviceNameDialog = true }
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MullvadWhite
            )
        }
        if (showDeviceNameDialog) {
            DeviceNameInfoDialog { showDeviceNameDialog = false }
        }
    }
}

@Composable
private fun PaymentPanel(
    showSitePayment: Boolean,
    billingPaymentState: PaymentState,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: String) -> Unit
) {
    Column(
        modifier =
            Modifier.fillMaxWidth()
                .padding(top = Dimens.mediumPadding)
                .background(color = MaterialTheme.colorScheme.background)
    ) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenVerticalMargin))
        PlayPaymentButton(
            billingPaymentState = billingPaymentState,
            onPurchaseBillingProductClick = onPurchaseBillingProductClick,
            modifier =
                Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    )
                    .align(Alignment.CenterHorizontally)
        )
        if (showSitePayment) {
            SitePaymentButton(
                onClick = onSitePaymentClick,
                isEnabled = true,
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.buttonSpacing
                    )
            )
        }
        RedeemVoucherButton(
            onClick = onRedeemVoucherClick,
            isEnabled = true,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin
                )
        )
    }
}
