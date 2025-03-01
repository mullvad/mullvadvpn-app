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
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.credentials.CreatePasswordRequest
import androidx.credentials.CredentialManager
import androidx.credentials.exceptions.CreateCredentialException
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import co.touchlab.kermit.Logger
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.AccountDestination
import com.ramcosta.composedestinations.generated.destinations.ConnectDestination
import com.ramcosta.composedestinations.generated.destinations.DeviceNameInfoDestination
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
import net.mullvad.mullvadvpn.compose.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.WelcomeScreenUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewWelcomeScreen(
    @PreviewParameter(WelcomeScreenUiStatePreviewParameterProvider::class) state: WelcomeUiState
) {
    AppTheme {
        WelcomeScreen(
            state = state,
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            onSettingsClick = {},
            onAccountClick = {},
            onPurchaseBillingProductClick = { _ -> },
            navigateToDeviceInfoDialog = {},
            navigateToVerificationPendingDialog = {},
            onDisconnectClick = {},
        )
    }
}

@Destination<RootGraph>(style = HomeTransition::class)
@Composable
fun Welcome(
    navigator: DestinationsNavigator,
    voucherRedeemResultRecipient: ResultRecipient<RedeemVoucherDestination, Boolean>,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>,
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

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()
    CollectSideEffectWithLifecycle(sideEffect = vm.uiSideEffect, Lifecycle.State.RESUMED) {
        uiSideEffect ->
        when (uiSideEffect) {
            is WelcomeViewModel.UiSideEffect.OpenAccountView -> openAccountPage(uiSideEffect.token)
            WelcomeViewModel.UiSideEffect.OpenConnectScreen ->
                navigator.navigate(ConnectDestination) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            WelcomeViewModel.UiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = context.getString(R.string.error_occurred)
                )
            is WelcomeViewModel.UiSideEffect.StoreCredentialsRequest -> {
                // UserId is not allowed to be empty
                val createPasswordRequest =
                    CreatePasswordRequest(id = "-", password = uiSideEffect.accountNumber.value)
                val credentialsManager = CredentialManager.create(context)
                try {
                    credentialsManager.createCredential(context, createPasswordRequest)
                } catch (e: CreateCredentialException) {
                    Logger.w("Unable to create Credentials")
                }
            }
        }
    }

    WelcomeScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSitePaymentClick = dropUnlessResumed { vm.onSitePaymentClick() },
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onAccountClick = dropUnlessResumed { navigator.navigate(AccountDestination) },
        navigateToDeviceInfoDialog =
            dropUnlessResumed { navigator.navigate(DeviceNameInfoDestination) },
        onPurchaseBillingProductClick =
            dropUnlessResumed { productId -> navigator.navigate(PaymentDestination(productId)) },
        onDisconnectClick = vm::onDisconnectClick,
        navigateToVerificationPendingDialog =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
    )
}

@Composable
fun WelcomeScreen(
    state: WelcomeUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit,
    onDisconnectClick: () -> Unit,
    navigateToDeviceInfoDialog: () -> Unit,
    navigateToVerificationPendingDialog: () -> Unit,
) {
    val scrollState = rememberScrollState()

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary,
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        snackbarHostState = snackbarHostState,
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .background(color = MaterialTheme.colorScheme.surface)
                    .padding(it)
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
        ) {
            // Welcome info area
            WelcomeInfo(snackbarHostState, state, navigateToDeviceInfoDialog)

            Spacer(modifier = Modifier.weight(1f))

            // Button area
            ButtonPanel(
                showDisconnectButton = state.tunnelState.isSecured(),
                showSitePayment = state.showSitePayment,
                billingPaymentState = state.billingPaymentState,
                onSitePaymentClick = onSitePaymentClick,
                onRedeemVoucherClick = onRedeemVoucherClick,
                onPurchaseBillingProductClick = onPurchaseBillingProductClick,
                onPaymentInfoClick = navigateToVerificationPendingDialog,
                onDisconnectClick = onDisconnectClick,
            )
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    state: WelcomeUiState,
    navigateToDeviceInfoDialog: () -> Unit,
) {
    Column {
        Text(
            text = stringResource(id = R.string.congrats),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        top = Dimens.screenVerticalMargin,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                    ),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = stringResource(id = R.string.here_is_your_account_number),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(horizontal = Dimens.sideMargin, vertical = Dimens.smallPadding),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface,
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
                    end = Dimens.sideMargin,
                ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun AccountNumberRow(snackbarHostState: SnackbarHostState, state: WelcomeUiState) {
    val copiedAccountNumberMessage = stringResource(id = R.string.copied_mullvad_account_number)
    val copyToClipboard = createCopyToClipboardHandle(snackbarHostState = snackbarHostState)
    val onCopyToClipboard = {
        copyToClipboard(state.accountNumber?.value ?: "", copiedAccountNumberMessage)
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier =
            Modifier.fillMaxWidth()
                .clickable(onClick = onCopyToClipboard)
                .padding(horizontal = Dimens.sideMargin),
    ) {
        Text(
            text = state.accountNumber?.value?.groupWithSpaces() ?: "",
            modifier = Modifier.weight(1f).padding(vertical = Dimens.smallPadding),
            style = MaterialTheme.typography.headlineSmall,
            color = MaterialTheme.colorScheme.onSurface,
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
            color = MaterialTheme.colorScheme.onSurface,
        )

        IconButton(
            modifier = Modifier.align(Alignment.CenterVertically),
            onClick = navigateToDeviceInfoDialog,
        ) {
            Icon(
                imageVector = Icons.Default.Info,
                contentDescription = null,
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
    }
}

@Composable
private fun ButtonPanel(
    showDisconnectButton: Boolean,
    showSitePayment: Boolean,
    billingPaymentState: PaymentState?,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit,
    onPaymentInfoClick: () -> Unit,
    onDisconnectClick: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth().padding(top = Dimens.mediumPadding)) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenVerticalMargin))
        if (showDisconnectButton) {
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
                            bottom = Dimens.buttonSpacing,
                        )
                        .align(Alignment.CenterHorizontally),
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
                        bottom = Dimens.buttonSpacing,
                    ),
            )
        }
        RedeemVoucherButton(
            onClick = onRedeemVoucherClick,
            isEnabled = true,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin,
                ),
        )
    }
}
