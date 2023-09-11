package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.theme.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel

@Preview
@Composable
private fun PreviewWelcomeScreen() {
    AppTheme {
        WelcomeScreen(
            showSitePayment = true,
            uiState = WelcomeUiState(accountNumber = "4444555566667777"),
            viewActions = MutableSharedFlow<WelcomeViewModel.ViewAction>().asSharedFlow(),
            onSitePaymentClick = {},
            onRedeemVoucherClick = {},
            onSettingsClick = {},
            onAccountClick = {},
            openConnectScreen = {}
        )
    }
}

@Composable
fun WelcomeScreen(
    showSitePayment: Boolean,
    uiState: WelcomeUiState,
    viewActions: SharedFlow<WelcomeViewModel.ViewAction>,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    openConnectScreen: () -> Unit
) {
    val context = LocalContext.current
    LaunchedEffect(Unit) {
        viewActions.collect { viewAction ->
            when (viewAction) {
                is WelcomeViewModel.ViewAction.OpenAccountView ->
                    context.openAccountPageInBrowser(viewAction.token)
                WelcomeViewModel.ViewAction.OpenConnectScreen -> openConnectScreen()
            }
        }
    }
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
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(scrollState)
                    .background(color = MaterialTheme.colorScheme.primary)
                    .padding(it)
        ) {
            // Welcome info area
            WelcomeInfo(snackbarHostState, uiState, showSitePayment)

            Spacer(modifier = Modifier.weight(1f))

            // Payment button area
            PaymentPanel(showSitePayment, onSitePaymentClick, onRedeemVoucherClick)
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    uiState: WelcomeUiState,
    showSitePayment: Boolean
) {
    Column(modifier = Modifier.padding(horizontal = Dimens.sideMargin)) {
        Text(
            text = stringResource(id = R.string.congrats),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        top = Dimens.screenVerticalMargin,
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
                        vertical = Dimens.smallPadding,
                    ),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onPrimary
        )

        AccountNumberRow(snackbarHostState, uiState)

        Text(
            text =
                buildString {
                    append(stringResource(id = R.string.pay_to_start_using))
                    if (showSitePayment) {
                        append(" ")
                        append(stringResource(id = R.string.add_time_to_account))
                    }
                },
            modifier = Modifier.padding(top = Dimens.smallPadding, bottom = Dimens.verticalSpace),
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
                .padding(start = Dimens.smallPadding)
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
private fun PaymentPanel(
    showSitePayment: Boolean,
    onSitePaymentClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit
) {
    Column(
        modifier =
            Modifier.fillMaxWidth()
                .padding(top = Dimens.mediumPadding)
                .background(color = MaterialTheme.colorScheme.background)
    ) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenVerticalMargin))
        if (showSitePayment) {
            ActionButton(
                onClick = onSitePaymentClick,
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
                colors =
                    ButtonDefaults.buttonColors(
                        contentColor = MaterialTheme.colorScheme.onPrimary,
                        containerColor = MaterialTheme.colorScheme.surface
                    )
            ) {
                Box(modifier = Modifier.fillMaxSize()) {
                    Text(
                        text = stringResource(id = R.string.buy_credit),
                        textAlign = TextAlign.Center,
                        fontSize = 18.sp,
                        fontWeight = FontWeight.Bold,
                        modifier = Modifier.align(Alignment.Center)
                    )
                    Image(
                        painter = painterResource(id = R.drawable.icon_extlink),
                        contentDescription = null,
                        modifier =
                            Modifier.align(Alignment.CenterEnd)
                                .padding(horizontal = Dimens.smallPadding)
                    )
                }
            }
        }

        ActionButton(
            text = stringResource(id = R.string.redeem_voucher),
            onClick = onRedeemVoucherClick,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin
                ),
            colors =
                ButtonDefaults.buttonColors(
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    containerColor = MaterialTheme.colorScheme.surface
                )
        )
    }
}
