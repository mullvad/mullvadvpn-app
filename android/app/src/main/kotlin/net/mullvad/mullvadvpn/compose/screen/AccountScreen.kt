package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Info
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.NavGraphs
import com.ramcosta.composedestinations.generated.destinations.DeviceNameInfoDestination
import com.ramcosta.composedestinations.generated.destinations.LoginDestination
import com.ramcosta.composedestinations.generated.destinations.PaymentDestination
import com.ramcosta.composedestinations.generated.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import java.time.ZonedDateTime
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ExternalButton
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.component.CopyableObfuscationView
import net.mullvad.mullvadvpn.compose.component.InformationView
import net.mullvad.mullvadvpn.compose.component.MissingPolicy
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.AccountUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.transitions.AccountTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.toExpiryDateString
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("PaymentAvailable|Loading|NoPayment|NoProductsFound|Error.Billing")
@Composable
private fun PreviewAccountScreen(
    @PreviewParameter(AccountUiStatePreviewParameterProvider::class) state: AccountUiState
) {
    AppTheme { AccountScreen(state = state, SnackbarHostState(), {}, {}, {}, {}, {}, {}, {}, {}) }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(style = AccountTransition::class)
@Composable
fun Account(
    navigator: DestinationsNavigator,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>,
) {
    val vm = koinViewModel<AccountViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val copyTextString = stringResource(id = R.string.copied_mullvad_account_number)
    val errorString = stringResource(id = R.string.error_occurred)
    val copyToClipboard = createCopyToClipboardHandle(snackbarHostState = snackbarHostState)
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            AccountViewModel.UiSideEffect.NavigateToLogin -> {
                navigator.navigate(LoginDestination(null)) {
                    launchSingleTop = true
                    popUpTo(NavGraphs.root) { inclusive = true }
                }
            }
            is AccountViewModel.UiSideEffect.OpenAccountManagementPageInBrowser ->
                openAccountPage(sideEffect.token)
            is AccountViewModel.UiSideEffect.CopyAccountNumber ->
                launch { copyToClipboard(sideEffect.accountNumber, copyTextString) }
            AccountViewModel.UiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(message = errorString)
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

    AccountScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onManageAccountClick = vm::onManageAccountClick,
        onLogoutClick = vm::onLogoutClick,
        onCopyAccountNumber = vm::onCopyAccountNumber,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        navigateToDeviceInfo = dropUnlessResumed { navigator.navigate(DeviceNameInfoDestination) },
        onPurchaseBillingProductClick =
            dropUnlessResumed { productId -> navigator.navigate(PaymentDestination(productId)) },
        navigateToVerificationPendingDialog =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
    )
}

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    state: AccountUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onCopyAccountNumber: (String) -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onManageAccountClick: () -> Unit,
    onLogoutClick: () -> Unit,
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit,
    navigateToDeviceInfo: () -> Unit,
    navigateToVerificationPendingDialog: () -> Unit,
    onBackClick: () -> Unit,
) {
    // This will enable SECURE_FLAG while this screen is visible to preview screenshot
    SecureScreenWhileInView()

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_account),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.Start,
            modifier =
                modifier
                    .animateContentSize()
                    .padding(horizontal = Dimens.sideMargin)
                    .padding(bottom = Dimens.screenVerticalMargin),
        ) {
            Column(
                verticalArrangement = Arrangement.spacedBy(Dimens.accountRowSpacing),
                modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize(),
            ) {
                DeviceNameRow(
                    deviceName = state.deviceName ?: "",
                    onInfoClick = navigateToDeviceInfo,
                )

                AccountNumberRow(
                    accountNumber = state.accountNumber?.value ?: "",
                    onCopyAccountNumber,
                )

                PaidUntilRow(accountExpiry = state.accountExpiry)
            }

            Spacer(modifier = Modifier.weight(1f))

            state.billingPaymentState?.let {
                PlayPayment(
                    billingPaymentState = state.billingPaymentState,
                    onPurchaseBillingProductClick = { productId ->
                        onPurchaseBillingProductClick(productId)
                    },
                    onInfoClick = navigateToVerificationPendingDialog,
                    modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                )
            }

            if (state.showSitePayment) {
                ExternalButton(
                    text = stringResource(id = R.string.manage_account),
                    onClick = onManageAccountClick,
                    modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                    isLoading = state.showManageAccountLoading,
                )
            }

            RedeemVoucherButton(
                onClick = onRedeemVoucherClick,
                modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                isEnabled = true,
            )

            NegativeButton(
                text = stringResource(id = R.string.log_out),
                onClick = onLogoutClick,
                isLoading = state.showLogoutLoading,
            )
        }
    }
}

@Composable
private fun DeviceNameRow(deviceName: String, onInfoClick: () -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.device_name),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            InformationView(content = deviceName, whenMissing = MissingPolicy.SHOW_SPINNER)
            IconButton(onClick = onInfoClick) {
                Icon(
                    imageVector = Icons.Default.Info,
                    contentDescription = stringResource(id = R.string.more_information),
                    tint = MaterialTheme.colorScheme.onSurface,
                )
            }
        }
    }
}

@Composable
private fun AccountNumberRow(accountNumber: String, onCopyAccountNumber: (String) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.account_number),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        CopyableObfuscationView(
            content = accountNumber,
            onCopyClicked = { onCopyAccountNumber(accountNumber) },
            modifier = Modifier.heightIn(min = Dimens.accountRowMinHeight).fillMaxWidth(),
        )
    }
}

@Composable
private fun PaidUntilRow(accountExpiry: ZonedDateTime?) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.paid_until),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        Row(
            modifier = Modifier.heightIn(min = Dimens.accountRowMinHeight),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            InformationView(
                content = accountExpiry?.toExpiryDateString() ?: "",
                whenMissing = MissingPolicy.SHOW_SPINNER,
            )
        }
    }
}
