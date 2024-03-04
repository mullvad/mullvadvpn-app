package net.mullvad.mullvadvpn.compose.screen

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
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.destinations.AccountDestination
import net.mullvad.mullvadvpn.compose.destinations.ConnectDestination
import net.mullvad.mullvadvpn.compose.destinations.DeviceNameInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.PaymentDestination
import net.mullvad.mullvadvpn.compose.destinations.RedeemVoucherDestination
import net.mullvad.mullvadvpn.compose.destinations.SettingsDestination
import net.mullvad.mullvadvpn.compose.destinations.VerificationPendingDialogDestination
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewWelcomeScreen() {
    AppTheme {
        WelcomeScreen(
            state =
                WelcomeUiState(
                    accountNumber = "4444555566667777",
                    deviceName = "Happy Mole",
                    billingPaymentState =
                        PaymentState.PaymentAvailable(
                            products =
                                listOf(
                                    PaymentProduct(ProductId("product"), ProductPrice("$44"), null)
                                )
                        )
                ),
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            onSettingsClick = {},
            onAccountClick = {},
            onPurchaseBillingProductClick = { _ -> },
            navigateToDeviceInfoDialog = {},
            navigateToVerificationPendingDialog = {}
        )
    }
}

@Destination(style = HomeTransition::class)
@Composable
fun Welcome(
    navigator: DestinationsNavigator,
    voucherRedeemResultRecipient: ResultRecipient<RedeemVoucherDestination, Boolean>,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>
) {
    val vm = koinViewModel<WelcomeViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    voucherRedeemResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value ->
                // If we successfully redeemed a voucher, navigate to Connect screen
                if (it.value) {
                    navigator.navigate(ConnectDestination) {
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
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

    val context = LocalContext.current
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                is WelcomeViewModel.UiSideEffect.OpenAccountView ->
                    context.openAccountPageInBrowser(uiSideEffect.token)
                WelcomeViewModel.UiSideEffect.OpenConnectScreen -> {
                    navigator.navigate(ConnectDestination) {
                        launchSingleTop = true
                        popUpTo(NavGraphs.root) { inclusive = true }
                    }
                }
            }
        }
    }

    WelcomeScreen(
        state = state,
        onSitePaymentClick = vm::onSitePaymentClick,
        onRedeemVoucherClick = {
            navigator.navigate(RedeemVoucherDestination) { launchSingleTop = true }
        },
        onSettingsClick = { navigator.navigate(SettingsDestination) { launchSingleTop = true } },
        onAccountClick = { navigator.navigate(AccountDestination) { launchSingleTop = true } },
        navigateToDeviceInfoDialog = {
            navigator.navigate(DeviceNameInfoDialogDestination) { launchSingleTop = true }
        },
        onPurchaseBillingProductClick = { productId ->
            navigator.navigate(PaymentDestination(productId)) { launchSingleTop = true }
        },
        navigateToVerificationPendingDialog = {
            navigator.navigate(VerificationPendingDialogDestination) { launchSingleTop = true }
        }
    )
}

@Composable
fun WelcomeScreen(
    state: WelcomeUiState,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit,
    navigateToDeviceInfoDialog: () -> Unit,
    navigateToVerificationPendingDialog: () -> Unit
) {
    val scrollState = rememberScrollState()
    val snackbarHostState = remember { SnackbarHostState() }

    ScaffoldWithTopBar(
        topBarColor =
            if (state.tunnelState.isSecured()) {
                MaterialTheme.colorScheme.inversePrimary
            } else {
                MaterialTheme.colorScheme.error
            },
        iconTintColor =
            if (state.tunnelState.isSecured()) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onError
                }
                .copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        snackbarHostState = snackbarHostState,
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
        ) {
            // Welcome info area
            WelcomeInfo(snackbarHostState, state, navigateToDeviceInfoDialog)

            Spacer(modifier = Modifier.weight(1f))

            // Payment button area
            PaymentPanel(
                showSitePayment = state.showSitePayment,
                billingPaymentState = state.billingPaymentState,
                onSitePaymentClick = onSitePaymentClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onPaymentInfoClick = navigateToVerificationPendingDialog
            )
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    state: WelcomeUiState,
    navigateToDeviceInfoDialog: () -> Unit
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

        AccountNumberRow(snackbarHostState, state)

        DeviceNameRow(deviceName = state.deviceName, navigateToDeviceInfoDialog)

        Text(
            text =
                buildString {
                    append(stringResource(id = R.string.pay_to_start_using))
                    if (state.showSitePayment) {
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
private fun AccountNumberRow(snackbarHostState: SnackbarHostState, state: WelcomeUiState) {
    val copiedAccountNumberMessage = stringResource(id = R.string.copied_mullvad_account_number)
    val copyToClipboard = createCopyToClipboardHandle(snackbarHostState = snackbarHostState)
    val onCopyToClipboard = {
        copyToClipboard(state.accountNumber ?: "", copiedAccountNumberMessage)
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
            text = state.accountNumber?.groupWithSpaces() ?: "",
            modifier = Modifier.weight(1f).padding(vertical = Dimens.smallPadding),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onPrimary
        )

        CopyAnimatedIconButton(onCopyToClipboard)
    }
}

@Composable
fun DeviceNameRow(deviceName: String?, navigateToDeviceInfoDialog: () -> Unit) {
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

        IconButton(
            modifier = Modifier.align(Alignment.CenterVertically),
            onClick = navigateToDeviceInfoDialog
        ) {
            Icon(
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onPrimary
            )
        }
    }
}

@Composable
private fun PaymentPanel(
    showSitePayment: Boolean,
    billingPaymentState: PaymentState?,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit,
    onPaymentInfoClick: () -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().padding(top = Dimens.mediumPadding)) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenVerticalMargin))
        billingPaymentState?.let {
            PlayPayment(
                billingPaymentState = billingPaymentState,
                onPurchaseBillingProductClick = { productId ->
                    onPurchaseBillingProductClick(productId)
                },
                onInfoClick = onPaymentInfoClick,
                modifier =
                    Modifier.padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            bottom = Dimens.buttonSpacing
                        )
                        .align(Alignment.CenterHorizontally)
            )
        }
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
