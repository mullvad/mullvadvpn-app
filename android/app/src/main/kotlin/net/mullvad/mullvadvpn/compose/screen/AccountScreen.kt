package net.mullvad.mullvadvpn.compose.screen

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
import androidx.compose.ui.tooling.preview.Preview
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ExternalButton
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.RedeemVoucherButton
import net.mullvad.mullvadvpn.compose.component.CopyableObfuscationView
import net.mullvad.mullvadvpn.compose.component.InformationView
import net.mullvad.mullvadvpn.compose.component.MissingPolicy
import net.mullvad.mullvadvpn.compose.component.NavigateBackDownIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.DeviceNameInfoDialog
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.common.util.openAccountPageInBrowser
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.toExpiryDateString
import net.mullvad.mullvadvpn.viewmodel.AccountUiState
import net.mullvad.mullvadvpn.viewmodel.AccountViewModel
import org.joda.time.DateTime

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAccountScreen() {
    AppTheme {
        AccountScreen(
            uiState =
                AccountUiState(
                    deviceName = "Test Name",
                    accountNumber = "1234123412341234",
                    accountExpiry = null
                ),
            uiSideEffect = MutableSharedFlow<AccountViewModel.UiSideEffect>().asSharedFlow(),
            enterTransitionEndAction = MutableSharedFlow()
        )
    }
}

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    uiState: AccountUiState,
    uiSideEffect: SharedFlow<AccountViewModel.UiSideEffect>,
    enterTransitionEndAction: SharedFlow<Unit>,
    onRedeemVoucherClick: () -> Unit = {},
    onManageAccountClick: () -> Unit = {},
    onLogoutClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    // This will enable SECURE_FLAG while this screen is visible to preview screenshot
    SecureScreenWhileInView()

    val context = LocalContext.current
    val backgroundColor = MaterialTheme.colorScheme.background
    val systemUiController = rememberSystemUiController()

    var showDeviceNameInfoDialog by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        systemUiController.setNavigationBarColor(backgroundColor)
        enterTransitionEndAction.collect { systemUiController.setStatusBarColor(backgroundColor) }
    }
    if (showDeviceNameInfoDialog) {
        DeviceNameInfoDialog { showDeviceNameInfoDialog = false }
    }

    LaunchedEffect(Unit) {
        uiSideEffect.collect { uiSideEffect ->
            if (uiSideEffect is AccountViewModel.UiSideEffect.OpenAccountManagementPageInBrowser) {
                context.openAccountPageInBrowser(uiSideEffect.token)
            }
        }
    }

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_account),
        navigationIcon = { NavigateBackDownIconButton(onBackClick) }
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.Start,
            verticalArrangement = Arrangement.spacedBy(Dimens.accountRowSpacing),
            modifier = modifier.animateContentSize().padding(horizontal = Dimens.sideMargin)
        ) {
            DeviceNameRow(deviceName = uiState.deviceName ?: "") { showDeviceNameInfoDialog = true }

            AccountNumberRow(accountNumber = uiState.accountNumber ?: "")

            PaidUntilRow(accountExpiry = uiState.accountExpiry)

            Spacer(modifier = Modifier.weight(1f))

            Column(modifier = Modifier.padding(bottom = Dimens.screenVerticalMargin)) {
                if (IS_PLAY_BUILD.not()) {
                    ExternalButton(
                        text = stringResource(id = R.string.manage_account),
                        onClick = onManageAccountClick,
                        modifier = Modifier.padding(bottom = Dimens.buttonSeparation)
                    )
                }

                RedeemVoucherButton(
                    onClick = onRedeemVoucherClick,
                    modifier = Modifier.padding(bottom = Dimens.buttonSeparation),
                    isEnabled = true
                )

                NegativeButton(
                    text = stringResource(id = R.string.log_out),
                    onClick = onLogoutClick,
                )
            }
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
private fun AccountNumberRow(accountNumber: String) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelMedium,
            text = stringResource(id = R.string.account_number),
        )
        CopyableObfuscationView(
            content = accountNumber,
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
