package net.mullvad.mullvadvpn.feature.account.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.DeleteForever
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.MoreVert
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuDefaults
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import java.time.ZonedDateTime
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.SecureScreenWhileInView
import net.mullvad.mullvadvpn.common.compose.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.common.compose.createOpenAccountPageHook
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.addtime.api.AddTimeNavKey
import net.mullvad.mullvadvpn.feature.addtime.api.VerificationPendingNavKey
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountNavKey
import net.mullvad.mullvadvpn.feature.login.api.LoginNavKey
import net.mullvad.mullvadvpn.feature.managedevices.api.ManageDevicesNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.util.toExpiryDateString
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeOutlinedButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryTextButton
import net.mullvad.mullvadvpn.lib.ui.tag.MANAGE_DEVICES_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaDisabled
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Content|LogoutLoading")
@Composable
private fun PreviewAccountScreen(
    @PreviewParameter(AccountUiStatePreviewParameterProvider::class) state: Lc<Unit, AccountUiState>
) {
    AppTheme {
        AccountScreen(
            state = state.contentOrNull(),
            snackbarHostState = SnackbarHostState(),
            onCopyAccountNumber = {},
            onManageDevicesClick = {},
            onLogoutClick = {},
            onPlayPaymentInfoClick = {},
            onBackClick = {},
            navigateToDeleteAccount = {},
            navigateToAddTime = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Account(navigator: Navigator) {
    val vm = koinViewModel<AccountViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val copyTextString = stringResource(id = R.string.copied_mullvad_account_number)
    val errorString = stringResource(id = R.string.error_occurred)
    val copyToClipboard =
        createCopyToClipboardHandle(snackbarHostState = snackbarHostState, isSensitive = true)
    val openAccountPage = LocalUriHandler.current.createOpenAccountPageHook()

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        when (sideEffect) {
            AccountViewModel.UiSideEffect.NavigateToLogin -> {
                navigator.navigate(LoginNavKey(), clearBackStack = true)
            }
            is AccountViewModel.UiSideEffect.OpenAccountManagementPageInBrowser ->
                openAccountPage(sideEffect.token)
            is AccountViewModel.UiSideEffect.CopyAccountNumber ->
                launch { copyToClipboard(sideEffect.accountNumber, copyTextString) }
            AccountViewModel.UiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(message = errorString)
        }
    }

    AccountScreen(
        state = state.contentOrNull(),
        snackbarHostState = snackbarHostState,
        onManageDevicesClick =
            dropUnlessResumed {
                state.contentOrNull()?.accountNumber?.let {
                    navigator.navigate(ManageDevicesNavKey(it))
                }
            },
        onLogoutClick = vm::onLogoutClick,
        onCopyAccountNumber = vm::onCopyAccountNumber,
        onPlayPaymentInfoClick =
            dropUnlessResumed { navigator.navigate(VerificationPendingNavKey) },
        onBackClick = dropUnlessResumed { navigator.goBack() },
        navigateToDeleteAccount = dropUnlessResumed { navigator.navigate(DeleteAccountNavKey) },
        navigateToAddTime = dropUnlessResumed { navigator.navigate(AddTimeNavKey) },
    )
}

@ExperimentalMaterial3Api
@Composable
fun AccountScreen(
    state: AccountUiState?,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onCopyAccountNumber: (String) -> Unit,
    onManageDevicesClick: () -> Unit,
    onLogoutClick: () -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    onBackClick: () -> Unit,
    navigateToDeleteAccount: () -> Unit,
    navigateToAddTime: () -> Unit,
) {
    // This will enable SECURE_FLAG while this screen is visible to preview screenshot
    SecureScreenWhileInView()

    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.settings_account),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
        snackbarHostState = snackbarHostState,
        actions = { AccountDropdownMenu(navigateToDeleteAccount) },
    ) { modifier ->
        Column(
            horizontalAlignment = Alignment.Start,
            modifier =
                modifier
                    .animateContentSize()
                    .padding(horizontal = Dimens.sideMargin)
                    .padding(bottom = Dimens.screenBottomMargin),
        ) {
            Column(
                verticalArrangement = Arrangement.spacedBy(Dimens.accountRowSpacing),
                modifier = Modifier.padding(bottom = Dimens.smallPadding).animateContentSize(),
            ) {
                DeviceNameRow(
                    deviceName = state?.deviceName ?: "",
                    onManageDevicesClick = onManageDevicesClick,
                )

                AccountNumberRow(
                    accountNumber = state?.accountNumber?.value ?: "",
                    onCopyAccountNumber,
                )

                PaidUntilRow(
                    accountExpiry = state?.accountExpiry,
                    verificationPending = state?.verificationPending == true,
                    onOpenPaymentScreen = navigateToAddTime,
                    onInfoClick = onPlayPaymentInfoClick,
                )
            }

            Spacer(modifier = Modifier.weight(1f))

            NegativeOutlinedButton(
                text = stringResource(id = R.string.log_out),
                onClick = onLogoutClick,
                isLoading = state?.showLogoutLoading == true,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun AccountDropdownMenu(navigateToDeleteAccount: () -> Unit) {
    var showMenu by remember { mutableStateOf(false) }

    IconButton(onClick = { showMenu = !showMenu }) {
        Icon(
            imageVector = Icons.Rounded.MoreVert,
            contentDescription = stringResource(R.string.more_actions),
        )
    }
    DropdownMenu(
        modifier = Modifier.background(MaterialTheme.colorScheme.tertiaryContainer),
        expanded = showMenu,
        onDismissRequest = { showMenu = false },
    ) {
        val colors =
            MenuDefaults.itemColors(
                leadingIconColor = MaterialTheme.colorScheme.onPrimary,
                disabledLeadingIconColor =
                    MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaDisabled),
            )

        DropdownMenuItem(
            text = { Text(text = stringResource(R.string.delete_account)) },
            onClick = {
                showMenu = false
                navigateToDeleteAccount()
            },
            colors = colors,
            leadingIcon = { Icon(Icons.Rounded.DeleteForever, contentDescription = null) },
        )
    }
}

@Composable
private fun DeviceNameRow(deviceName: String, onManageDevicesClick: () -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelLarge,
            text = stringResource(id = R.string.device_name),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        // Device name is english so always provide LtR direction
        CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                InformationView(content = deviceName, whenMissing = MissingPolicy.SHOW_SPINNER)
                Spacer(modifier = Modifier.weight(1f))
                PrimaryTextButton(
                    modifier = Modifier.testTag(MANAGE_DEVICES_BUTTON_TEST_TAG),
                    onClick = onManageDevicesClick,
                    text = stringResource(R.string.manage_devices),
                    textDecoration = TextDecoration.Underline,
                )
            }
        }
    }
}

@Composable
private fun AccountNumberRow(accountNumber: String, onCopyAccountNumber: (String) -> Unit) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelLarge,
            text = stringResource(id = R.string.account_number),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        // Always provide LtR direction since it is a number
        CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
            CopyableObfuscationView(
                content = accountNumber,
                onCopyClicked = { onCopyAccountNumber(accountNumber) },
                modifier = Modifier.heightIn(min = Dimens.accountRowMinHeight).fillMaxWidth(),
            )
        }
    }
}

@Composable
private fun PaidUntilRow(
    accountExpiry: ZonedDateTime?,
    verificationPending: Boolean,
    onOpenPaymentScreen: () -> Unit,
    onInfoClick: () -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth()) {
        Text(
            style = MaterialTheme.typography.labelLarge,
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
            Spacer(modifier = Modifier.weight(1f))
            PrimaryTextButton(
                onClick = onOpenPaymentScreen,
                text = stringResource(R.string.add_time),
                textDecoration = TextDecoration.Underline,
            )
        }

        if (verificationPending) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = onInfoClick) {
                    Icon(
                        imageVector = Icons.Rounded.Info,
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.onSurface,
                    )
                }
                Text(
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurface,
                    text = stringResource(R.string.payment_status_pending_short),
                )
            }
        }
    }
}
