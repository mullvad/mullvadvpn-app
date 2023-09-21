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
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
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
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.button.SitePaymentButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.InfoDialog
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.lib.common.util.SdkUtils
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.theme.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.MullvadWhite
import net.mullvad.mullvadvpn.ui.extension.copyToClipboard
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel

@Preview
@Composable
private fun PreviewWelcomeScreen() {
    AppTheme {
        WelcomeScreen(
            showSitePayment = true,
            uiState = WelcomeUiState(accountNumber = "4444555566667777", deviceName = "Happy Mole"),
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
    LaunchedEffect(key1 = Unit) {
        viewActions.collect { viewAction ->
            when (viewAction) {
                is WelcomeViewModel.ViewAction.OpenAccountView ->
                    context.openAccountPageInBrowser(viewAction.token)
                WelcomeViewModel.ViewAction.OpenConnectScreen -> openConnectScreen()
            }
        }
    }
    val scrollState = rememberScrollState()
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
        onAccountClicked = onAccountClick
    ) {
        Column(
            verticalArrangement = Arrangement.Bottom,
            horizontalAlignment = Alignment.Start,
            modifier =
                Modifier.fillMaxSize()
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(scrollState)
                    .background(color = MaterialTheme.colorScheme.primary)
                    .padding(it)
        ) {
            Text(
                text = stringResource(id = R.string.congrats),
                modifier =
                    Modifier.padding(
                        top = Dimens.screenVerticalMargin,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin
                    ),
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary
            )
            Text(
                text = stringResource(id = R.string.here_is_your_account_number),
                modifier =
                    Modifier.padding(
                        vertical = Dimens.smallPadding,
                        horizontal = Dimens.sideMargin
                    ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimary
            )
            Text(
                text = uiState.accountNumber?.groupWithSpaces() ?: "",
                modifier =
                    Modifier.fillMaxWidth()
                        .wrapContentHeight()
                        .then(
                            uiState.accountNumber?.let {
                                Modifier.clickable {
                                    context.copyToClipboard(
                                        content = uiState.accountNumber,
                                        clipboardLabel =
                                            context.getString(R.string.mullvad_account_number)
                                    )
                                    SdkUtils.showCopyToastIfNeeded(
                                        context,
                                        context.getString(R.string.copied_mullvad_account_number)
                                    )
                                }
                            }
                                ?: Modifier
                        )
                        .padding(vertical = Dimens.smallPadding, horizontal = Dimens.sideMargin),
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onPrimary
            )
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
                            append(uiState.deviceName)
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
                    InfoDialog(
                        message =
                            buildString {
                                appendLine(
                                    stringResource(id = R.string.device_name_info_first_paragraph)
                                )
                                appendLine()
                                appendLine(
                                    stringResource(id = R.string.device_name_info_second_paragraph)
                                )
                                appendLine()
                                appendLine(
                                    stringResource(id = R.string.device_name_info_third_paragraph)
                                )
                            },
                        onDismiss = { showDeviceNameDialog = false }
                    )
                }
            }
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
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.verticalSpace
                    ),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onPrimary
            )
            Spacer(modifier = Modifier.weight(1f))
            // Payment button area
            Column(
                modifier =
                    Modifier.fillMaxWidth()
                        .padding(top = Dimens.mediumPadding)
                        .background(color = MaterialTheme.colorScheme.background)
            ) {
                Spacer(modifier = Modifier.padding(top = Dimens.screenVerticalMargin))
                if (showSitePayment) {
                    SitePaymentButton(
                        onClick = onSitePaymentClick,
                        isEnabled = true,
                        modifier =
                            Modifier.padding(
                                start = Dimens.sideMargin,
                                end = Dimens.sideMargin,
                                bottom = Dimens.screenVerticalMargin
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
    }
}
