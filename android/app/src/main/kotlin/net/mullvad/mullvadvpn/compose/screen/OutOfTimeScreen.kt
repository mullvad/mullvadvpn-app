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
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.AccountDestination
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.PaymentDestination
import com.ramcosta.composedestinations.generated.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.destinations.SettingsDestination
import com.ramcosta.composedestinations.generated.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBarAndDeviceName
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.OutOfTimeScreenPreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.test.OUT_OF_TIME_SCREEN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Disconnected|Connecting|Error")
@Composable
private fun PreviewOutOfTimeScreen(
    @PreviewParameter(OutOfTimeScreenPreviewParameterProvider::class) state: OutOfTimeUiState
) {
    AppTheme { OutOfTimeScreen(state = state, SnackbarHostState(), {}, {}, {}, {}, {}, {}, {}) }
}

@Destination<RootGraph>(style = HomeTransition::class)
@Composable
fun OutOfTime(
    navigator: DestinationsNavigator,
    redeemVoucherResultRecipient: ResultRecipient<RedeemVoucherDestination, Boolean>,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>,
) {
    val vm = koinViewModel<OutOfTimeViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
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

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    CollectSideEffectWithLifecycle(vm.uiSideEffect, Lifecycle.State.RESUMED) { uiSideEffect ->
        when (uiSideEffect) {
            is OutOfTimeViewModel.UiSideEffect.OpenAccountView ->
                openAccountPage(uiSideEffect.token)
            OutOfTimeViewModel.UiSideEffect.OpenConnectScreen ->
                navigator.navigate(ConnectDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            OutOfTimeViewModel.UiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = context.getString(R.string.error_occurred)
                )
        }
    }

    OutOfTimeScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSitePaymentClick = vm::onSitePaymentClick,
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onAccountClick = dropUnlessResumed { navigator.navigate(AccountDestination) },
        onDisconnectClick = vm::onDisconnectClick,
        onPurchaseBillingProductClick =
            dropUnlessResumed { productId -> navigator.navigate(PaymentDestination(productId)) },
        navigateToVerificationPendingDialog =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
    )
}

@Composable
fun OutOfTimeScreen(
    state: OutOfTimeUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onDisconnectClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    navigateToVerificationPendingDialog: () -> Unit,
) {

    val scrollState = rememberScrollState()
    ScaffoldWithTopBarAndDeviceName(
        snackbarHostState = snackbarHostState,
        topBarColor =
            if (state.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.tertiary
            } else {
                MaterialTheme.colorScheme.error
            },
        iconTintColor =
            if (state.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.onTertiary
            } else {
                MaterialTheme.colorScheme.onError
            },
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        deviceName = state.deviceName,
        timeLeft = null,
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .background(color = MaterialTheme.colorScheme.surface)
        ) {
            Image(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(vertical = Dimens.screenVerticalMargin)
                        .size(Dimens.bigIconSize),
            )
            Text(
                text = stringResource(id = R.string.out_of_time),
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onSurface,
                modifier =
                    Modifier.padding(horizontal = Dimens.sideMargin)
                        .testTag(OUT_OF_TIME_SCREEN_TITLE_TEST_TAG),
            )
            Text(
                text =
                    buildString {
                        append(stringResource(R.string.account_credit_has_expired))
                        if (state.showSitePayment) {
                            append(" ")
                            append(stringResource(R.string.add_time_to_account))
                        }
                    },
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurface,
                modifier =
                    Modifier.padding(
                        top = Dimens.mediumPadding,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                    ),
            )
            Spacer(modifier = Modifier.weight(1f).defaultMinSize(minHeight = Dimens.verticalSpace))
            // Button area

            ButtonPanel(
                state = state,
                onDisconnectClick = onDisconnectClick,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onSitePaymentClick = onSitePaymentClick,
                navigateToVerificationPendingDialog = navigateToVerificationPendingDialog,
            )
        }
    }
}

@Composable
private fun ButtonPanel(
    state: OutOfTimeUiState,
    onDisconnectClick: () -> Unit,
    onPurchaseBillingProductClick: (ProductId) -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSitePaymentClick: () -> Unit,
    navigateToVerificationPendingDialog: () -> Unit,
) {

    Column {
        if (state.tunnelState.isSecured()) {
            NegativeButton(
                onClick = onDisconnectClick,
                text = stringResource(id = R.string.disconnect),
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.buttonSpacing,
                    ),
            )
        }
        state.billingPaymentState?.let {
            PlayPayment(
                billingPaymentState = state.billingPaymentState,
                onPurchaseBillingProductClick = { productId ->
                    onPurchaseBillingProductClick(productId)
                },
                onInfoClick = navigateToVerificationPendingDialog,
                modifier =
                    Modifier.padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            bottom = Dimens.buttonSpacing,
                        )
                        .align(Alignment.CenterHorizontally),
            )
        }
        if (state.showSitePayment) {
            SitePaymentButton(
                onClick = onSitePaymentClick,
                isEnabled = state.tunnelState.enableSitePaymentButton(),
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.buttonSpacing,
                    ),
            )
        }
        RedeemVoucherButton(
            onClick = onRedeemVoucherClick,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin,
                ),
            isEnabled = state.tunnelState.enableRedeemButton(),
        )
    }
}

private fun TunnelState.enableSitePaymentButton(): Boolean = this is TunnelState.Disconnected

private fun TunnelState.enableRedeemButton(): Boolean =
    !(this is TunnelState.Error && this.errorState.cause is ErrorStateCause.IsOffline)
