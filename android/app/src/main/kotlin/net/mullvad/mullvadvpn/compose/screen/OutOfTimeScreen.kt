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
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBarAndDeviceName
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.destinations.AccountDestination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.PaymentDestination
import net.mullvad.mullvadvpn.compose.destinations.RedeemVoucherDestination
import net.mullvad.mullvadvpn.compose.destinations.SettingsDestination
import net.mullvad.mullvadvpn.compose.destinations.VerificationPendingDialogDestination
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewOutOfTimeScreenDisconnected() {
    AppTheme {
        OutOfTimeScreen(
            uiState =
                OutOfTimeUiState(
                    tunnelState = TunnelState.Disconnected(),
                    "Heroic Frog",
                    showSitePayment = true
                ),
        )
    }
}

@Preview
@Composable
private fun PreviewOutOfTimeScreenConnecting() {
    AppTheme {
        OutOfTimeScreen(
            uiState =
                OutOfTimeUiState(
                    tunnelState = TunnelState.Connecting(null, null),
                    "Strong Rabbit",
                    showSitePayment = true
                ),
        )
    }
}

@Preview
@Composable
private fun PreviewOutOfTimeScreenError() {
    AppTheme {
        OutOfTimeScreen(
            uiState =
                OutOfTimeUiState(
                    tunnelState =
                        TunnelState.Error(
                            ErrorState(cause = ErrorStateCause.IsOffline, isBlocking = true)
                        ),
                    deviceName = "Stable Horse",
                    showSitePayment = true
                ),
        )
    }
}

@Destination(style = HomeTransition::class)
@Composable
fun OutOfTime(
    navigator: DestinationsNavigator,
    redeemVoucherResultRecipient: ResultRecipient<RedeemVoucherDestination, Boolean>,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>
) {
    val vm = koinViewModel<OutOfTimeViewModel>()
    val state = vm.uiState.collectAsState().value
    redeemVoucherResultRecipient.onNavResult {
        // If we successfully redeemed a voucher, navigate to Connect screen
        if (it is NavResult.Value && it.value) {
            navigator.navigate(ConnectDestination) {
                launchSingleTop = true
                popUpTo(NavGraphs.root) { inclusive = true }
            }
        }
    }

    playPaymentResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> vm.onClosePurchaseResultDialog(it.value)
        }
    }

    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                is OutOfTimeViewModel.UiSideEffect.OpenAccountView ->
                    openAccountPage(uiSideEffect.token)
                OutOfTimeViewModel.UiSideEffect.OpenConnectScreen -> {
                    navigator.navigate(ConnectDestination) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
            }
        }
    }

    OutOfTimeScreen(
        uiState = state,
        onSitePaymentClick = vm::onSitePaymentClick,
        onRedeemVoucherClick = {
            navigator.navigate(RedeemVoucherDestination) { launchSingleTop = true }
        },
        onSettingsClick = { navigator.navigate(SettingsDestination) { launchSingleTop = true } },
        onAccountClick = { navigator.navigate(AccountDestination) { launchSingleTop = true } },
        onDisconnectClick = vm::onDisconnectClick,
        onPurchaseBillingProductClick = { productId ->
            navigator.navigate(PaymentDestination(productId)) { launchSingleTop = true }
        },
        navigateToVerificationPendingDialog = {
            navigator.navigate(VerificationPendingDialogDestination) { launchSingleTop = true }
        }
    )
}

@Composable
fun OutOfTimeScreen(
    uiState: OutOfTimeUiState,
    onDisconnectClick: () -> Unit = {},
    onSitePaymentClick: () -> Unit = {},
    onRedeemVoucherClick: () -> Unit = {},
    onSettingsClick: () -> Unit = {},
    onAccountClick: () -> Unit = {},
    onPurchaseBillingProductClick: (ProductId) -> Unit = { _ -> },
    navigateToVerificationPendingDialog: () -> Unit = {}
) {

    val scrollState = rememberScrollState()
    ScaffoldWithTopBarAndDeviceName(
        topBarColor =
            if (uiState.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.inversePrimary
            } else {
                MaterialTheme.colorScheme.error
            },
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
                        if (uiState.showSitePayment) {
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
            uiState.billingPaymentState?.let {
                PlayPayment(
                    billingPaymentState = uiState.billingPaymentState,
                    onPurchaseBillingProductClick = { productId ->
                        onPurchaseBillingProductClick(productId)
                    },
                    onInfoClick = navigateToVerificationPendingDialog,
                    modifier =
                        Modifier.padding(
                                start = Dimens.sideMargin,
                                end = Dimens.sideMargin,
                                bottom = Dimens.buttonSpacing
                            )
                            .align(Alignment.CenterHorizontally)
                )
            }
            if (uiState.showSitePayment) {
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
