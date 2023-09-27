package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.PlayPaymentButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBarAndDeviceName
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.PaymentAvailabilityDialog
import net.mullvad.mullvadvpn.compose.dialog.PurchaseResultDialog
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause

@Preview
@Composable
private fun PreviewOutOfTimeScreenDisconnected() {
    AppTheme {
        OutOfTimeScreen(
            showSitePayment = true,
            uiState = OutOfTimeUiState(tunnelState = TunnelState.Disconnected, "Heroic Frog"),
            uiSideEffect = MutableSharedFlow<OutOfTimeViewModel.UiSideEffect>().asSharedFlow()
        )
    }
}

@Preview
@Composable
private fun PreviewOutOfTimeScreenConnecting() {
    AppTheme {
        OutOfTimeScreen(
            showSitePayment = true,
            uiState =
                OutOfTimeUiState(tunnelState = TunnelState.Connecting(null, null), "Strong Rabbit"),
            uiSideEffect = MutableSharedFlow<OutOfTimeViewModel.UiSideEffect>().asSharedFlow()
        )
    }
}

@Preview
@Composable
private fun PreviewOutOfTimeScreenError() {
    AppTheme {
        OutOfTimeScreen(
            showSitePayment = true,
            uiState =
                OutOfTimeUiState(
                    tunnelState =
                        TunnelState.Error(
                            ErrorState(cause = ErrorStateCause.IsOffline, isBlocking = true)
                        ),
                    deviceName = "Stable Horse"
                ),
            uiSideEffect = MutableSharedFlow<OutOfTimeViewModel.UiSideEffect>().asSharedFlow()
        )
    }
}

@Composable
fun OutOfTimeScreen(
    showSitePayment: Boolean,
    uiState: OutOfTimeUiState,
    uiSideEffect: SharedFlow<OutOfTimeViewModel.UiSideEffect>,
    onDisconnectClick: () -> Unit = {},
    onSitePaymentClick: () -> Unit = {},
    onRedeemVoucherClick: () -> Unit = {},
    openConnectScreen: () -> Unit = {},
    onSettingsClick: () -> Unit = {},
    onAccountClick: () -> Unit = {},
    onPurchaseBillingProductClick: (String) -> Unit = {},
    onTryFetchProductsAgain: () -> Unit = {},
    onTryVerificationAgain: () -> Unit = {},
    onClosePurchaseResultDialog: (success: Boolean) -> Unit = {}
) {
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    LaunchedEffect(key1 = Unit) {
        uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                is OutOfTimeViewModel.UiSideEffect.OpenAccountView ->
                    openAccountPage(uiSideEffect.token)
                OutOfTimeViewModel.UiSideEffect.OpenConnectScreen -> openConnectScreen()
            }
        }
    }

    uiState.purchaseResult?.let {
        PurchaseResultDialog(
            purchaseResult = uiState.purchaseResult,
            onTryAgain = onTryVerificationAgain,
            onCloseDialog = onClosePurchaseResultDialog
        )
    }

    PaymentAvailabilityDialog(
        paymentAvailability = uiState.billingPaymentState,
        onTryAgain = onTryFetchProductsAgain
    )

    val scrollState = rememberScrollState()
    ScaffoldWithTopBarAndDeviceName(
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
        deviceName = uiState.deviceName,
        timeLeft = null
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar)
                    )
                    .background(color = MaterialTheme.colorScheme.background)
        ) {
            Image(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(vertical = Dimens.screenVerticalMargin)
                        .size(Dimens.bigIconSize)
            )
            Text(
                text = stringResource(id = R.string.out_of_time),
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier.padding(horizontal = Dimens.sideMargin)
            )
            Text(
                text =
                    buildString {
                        append(stringResource(R.string.account_credit_has_expired))
                        if (showSitePayment) {
                            append(" ")
                            append(stringResource(R.string.add_time_to_account))
                        }
                    },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.padding(
                        top = Dimens.mediumPadding,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin
                    )
            )
            Spacer(modifier = Modifier.weight(1f).defaultMinSize(minHeight = Dimens.verticalSpace))
            // Button area
            if (uiState.tunnelState.showDisconnectButton()) {
                NegativeButton(
                    onClick = onDisconnectClick,
                    text = stringResource(id = R.string.disconnect),
                    modifier =
                        Modifier.padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            bottom = Dimens.buttonSpacing
                        )
                )
            }
            PlayPaymentButton(
                billingPaymentState = uiState.billingPaymentState,
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
                    isEnabled = uiState.tunnelState.enableSitePaymentButton(),
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
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
                isEnabled = uiState.tunnelState.enableRedeemButton()
            )
        }
    }
}

private fun TunnelState.showDisconnectButton(): Boolean =
    when (this) {
        is TunnelState.Disconnected -> false
        is TunnelState.Connecting,
        is TunnelState.Connected -> true
        is TunnelState.Disconnecting -> {
            this.actionAfterDisconnect != ActionAfterDisconnect.Nothing
        }
        is TunnelState.Error -> this.errorState.isBlocking
    }

private fun TunnelState.enableSitePaymentButton(): Boolean = this is TunnelState.Disconnected

private fun TunnelState.enableRedeemButton(): Boolean =
    !(this is TunnelState.Error && this.errorState.cause is ErrorStateCause.IsOffline)
