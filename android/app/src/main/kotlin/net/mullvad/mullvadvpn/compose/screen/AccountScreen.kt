package net.mullvad.mullvadvpn.compose.screen

import android.os.Build
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.navigation.popUpTo
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.NavGraphs
import net.mullvad.mullvadvpn.compose.button.ExternalButton
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.component.CopyableObfuscationView
import net.mullvad.mullvadvpn.compose.component.InformationView
import net.mullvad.mullvadvpn.compose.component.MissingPolicy
import net.mullvad.mullvadvpn.compose.component.NavigateBackDownIconButton
import net.mullvad.mullvadvpn.compose.component.PlayPayment
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.destinations.DeviceNameInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.LoginDestination
import net.mullvad.mullvadvpn.compose.destinations.PaymentDestination
import net.mullvad.mullvadvpn.compose.destinations.RedeemVoucherDestination
import net.mullvad.mullvadvpn.compose.destinations.VerificationPendingDialogDestination
import net.mullvad.mullvadvpn.compose.state.PaymentState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromBottomTransition
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.payment.model.PaymentProduct
import net.mullvad.mullvadvpn.lib.payment.model.PaymentStatus
import net.mullvad.mullvadvpn.lib.payment.model.ProductId
import net.mullvad.mullvadvpn.lib.payment.model.ProductPrice
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.toExpiryDateString
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.joda.time.DateTime
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAccountScreen() {
    AppTheme {
        AccountScreen(
            state =
                AccountUiState(
                    deviceName = "Test Name",
                    accountNumber = "1234123412341234",
                    accountExpiry = null,
                    showSitePayment = true,
                    billingPaymentState =
                        PaymentState.PaymentAvailable(
                            listOf(
                                PaymentProduct(
                                    ProductId("productId"),
                                    price = ProductPrice("34 SEK"),
                                    status = null
                                ),
                                PaymentProduct(
                                    ProductId("productId_pending"),
                                    price = ProductPrice("34 SEK"),
                                    status = PaymentStatus.PENDING
                                )
                            ),
                        )
                ),
            uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination(style = SlideInFromBottomTransition::class)
@Composable
fun Account(
    navigator: DestinationsNavigator,
    playPaymentResultRecipient: ResultRecipient<PaymentDestination, Boolean>
) {
    val vm = koinViewModel<AccountViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

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
        uiSideEffect = vm.uiSideEffect,
        onRedeemVoucherClick = {
            navigator.navigate(RedeemVoucherDestination) { launchSingleTop = true }
        },
        onManageAccountClick = vm::onManageAccountClick,
        onLogoutClick = vm::onLogoutClick,
        navigateToLogin = {
            navigator.navigate(LoginDestination(null)) {
                launchSingleTop = true
                popUpTo(NavGraphs.root) { inclusive = true }
            }
        },
        onCopyAccountNumber = vm::onCopyAccountNumber,
        onBackClick = navigator::navigateUp,
        navigateToDeviceInfo = {
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

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    state: AccountUiState,
    uiSideEffect: Flow<AccountViewModel.UiSideEffect>,
    onCopyAccountNumber: (String) -> Unit = {},
    onRedeemVoucherClick: () -> Unit = {},
    onManageAccountClick: () -> Unit = {},
    onLogoutClick: () -> Unit = {},
    onPurchaseBillingProductClick: (productId: ProductId) -> Unit = { _ -> },
    navigateToLogin: () -> Unit = {},
    navigateToDeviceInfo: () -> Unit = {},
    navigateToVerificationPendingDialog: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    // This will enable SECURE_FLAG while this screen is visible to preview screenshot
    SecureScreenWhileInView()

    val context = LocalContext.current
    val clipboardManager = LocalClipboardManager.current
    val snackbarHostState = remember { SnackbarHostState() }
    val copyTextString = stringResource(id = R.string.copied_mullvad_account_number)
    LaunchedEffect(Unit) {
        uiSideEffect.collect { uiSideEffect ->
            when (uiSideEffect) {
                AccountViewModel.UiSideEffect.NavigateToLogin -> navigateToLogin()
                is AccountViewModel.UiSideEffect.OpenAccountManagementPageInBrowser ->
                    context.openAccountPageInBrowser(uiSideEffect.token)
                is AccountViewModel.UiSideEffect.CopyAccountNumber ->
                    launch {
                        clipboardManager.setText(AnnotatedString(uiSideEffect.accountNumber))

                        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
                            snackbarHostState.currentSnackbarData?.dismiss()
                            snackbarHostState.showSnackbar(message = copyTextString)
                        }
                    }
            }
        }
    }

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_account),
        navigationIcon = { NavigateBackDownIconButton(onBackClick) }
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.Start,
            modifier =
                modifier
                    .animateContentSize()
                    .padding(horizontal = Dimens.sideMargin)
                    .padding(bottom = Dimens.screenVerticalMargin)
        ) {
            Column(
                verticalArrangement = Arrangement.spacedBy(Dimens.accountRowSpacing),
                modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize()
            ) {
                DeviceNameRow(
                    deviceName = state.deviceName ?: "",
                    onInfoClick = navigateToDeviceInfo
                )

                AccountNumberRow(accountNumber = state.accountNumber ?: "", onCopyAccountNumber)

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
                    modifier = Modifier.padding(bottom = Dimens.buttonSpacing)
                )
            }

            if (state.showSitePayment) {
                ExternalButton(
                    text = stringResource(id = R.string.manage_account),
                    onClick = onManageAccountClick,
                    modifier = Modifier.padding(bottom = Dimens.buttonSpacing)
                )
            }

            RedeemVoucherButton(
                onClick = onRedeemVoucherClick,
                modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
                isEnabled = true
            )

            NegativeButton(
                text = stringResource(id = R.string.log_out),
                onClick = onLogoutClick,
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
        )

        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
            InformationView(content = deviceName, whenMissing = MissingPolicy.SHOW_SPINNER)
            IconButton(onClick = onInfoClick) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_info),
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.inverseSurface
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
        )
        CopyableObfuscationView(
            content = accountNumber,
            onCopyClicked = { onCopyAccountNumber(accountNumber) },
            modifier = Modifier.heightIn(min = Dimens.accountRowMinHeight).fillMaxWidth()
        )
    }
}

@Composable
private fun PaidUntilRow(accountExpiry: DateTime?) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.paid_until),
        )

        Row(
            modifier = Modifier.heightIn(min = Dimens.accountRowMinHeight),
            verticalAlignment = Alignment.CenterVertically
        ) {
            InformationView(
                content = accountExpiry?.toExpiryDateString() ?: "",
                whenMissing = MissingPolicy.SHOW_SPINNER
            )
        }
    }
}
