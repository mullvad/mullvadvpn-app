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
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
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
import com.ramcosta.composedestinations.generated.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.destinations.SettingsDestination
import com.ramcosta.composedestinations.generated.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.AddTimeBottomSheet
import net.mullvad.mullvadvpn.compose.component.CopyAnimatedIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.preview.WelcomeScreenUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.WelcomeUiState
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.groupWithSpaces
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.viewmodel.WelcomeViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Loading|Content|TunnelConnected")
@Composable
private fun PreviewWelcomeScreen(
    @PreviewParameter(WelcomeScreenUiStatePreviewParameterProvider::class)
    state: Lc<Unit, WelcomeUiState>
) {
    AppTheme {
        WelcomeScreen(
            state = state,
            onSettingsClick = {},
            onAccountClick = {},
            navigateToDeviceInfoDialog = {},
            onDisconnectClick = {},
            onRedeemVoucherClick = {},
            onPlayPaymentInfoClick = {},
        )
    }
}

@Destination<RootGraph>(style = HomeTransition::class)
@Composable
fun Welcome(navigator: DestinationsNavigator) {
    val vm = koinViewModel<WelcomeViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val resources = LocalResources.current
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
                    message = resources.getString(R.string.error_occurred)
                )
            is WelcomeViewModel.UiSideEffect.StoreCredentialsRequest -> {
                // UserId is not allowed to be empty
                val createPasswordRequest =
                    CreatePasswordRequest(id = "-", password = uiSideEffect.accountNumber.value)
                val credentialsManager = CredentialManager.create(context)
                try {
                    credentialsManager.createCredential(context, createPasswordRequest)
                } catch (_: CreateCredentialException) {
                    Logger.w("Unable to create Credentials")
                }
            }
        }
    }

    WelcomeScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onAccountClick = dropUnlessResumed { navigator.navigate(AccountDestination) },
        navigateToDeviceInfoDialog =
            dropUnlessResumed { navigator.navigate(DeviceNameInfoDestination) },
        onDisconnectClick = vm::onDisconnectClick,
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onPlayPaymentInfoClick =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
    )
}

@Composable
fun WelcomeScreen(
    state: Lc<Unit, WelcomeUiState>,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onDisconnectClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
    navigateToDeviceInfoDialog: () -> Unit,
) {
    val scrollState = rememberScrollState()

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary,
        onSettingsClicked = onSettingsClick,
        onAccountClicked = onAccountClick,
        snackbarHostState = snackbarHostState,
    ) {
        var addTimeBottomSheetState by remember { mutableStateOf(false) }
        if (!LocalInspectionMode.current) {
            AddTimeBottomSheet(
                visible = addTimeBottomSheetState,
                onHideBottomSheet = { addTimeBottomSheetState = false },
                onRedeemVoucherClick = onRedeemVoucherClick,
                onPlayPaymentInfoClick = onPlayPaymentInfoClick,
            )
        }

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
            if (state is Lc.Content) {
                ButtonPanel(
                    showDisconnectButton = state.value.tunnelState.isSecured(),
                    verificationPending = state.value.verificationPending,
                    onAddMoreTimeClick = { addTimeBottomSheetState = true },
                    onDisconnectClick = onDisconnectClick,
                    onInfoClick = onPlayPaymentInfoClick,
                )
            }
        }
    }
}

@Composable
private fun WelcomeInfo(
    snackbarHostState: SnackbarHostState,
    state: Lc<Unit, WelcomeUiState>,
    navigateToDeviceInfoDialog: () -> Unit,
) {
    Column {
        Text(
            text = stringResource(id = R.string.congrats),
            modifier =
                Modifier.fillMaxWidth()
                    .padding(
                        top = Dimens.screenTopMargin,
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
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )

        when (state) {
            is Lc.Loading ->
                MullvadCircularProgressIndicatorMedium(
                    modifier =
                        Modifier.padding(
                            horizontal = Dimens.sideMargin,
                            vertical = Dimens.smallPadding,
                        )
                )
            is Lc.Content -> {
                // Content is English or numbers so we should keep Ltr direction.
                CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
                    // Account number
                    AccountNumberRow(snackbarHostState, state.value)
                }
                DeviceNameRow(deviceName = state.value.deviceName, navigateToDeviceInfoDialog)
            }
        }

        Text(
            text =
                buildString {
                    append(stringResource(id = R.string.pay_to_start_using))
                    if (state.contentOrNull()?.showSitePayment == true) {
                        append(" ")
                        append(stringResource(id = R.string.add_time_to_account))
                    }
                },
            modifier =
                Modifier.padding(
                    top = Dimens.cellVerticalSpacing,
                    bottom = Dimens.verticalSpace,
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                ),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun AccountNumberRow(snackbarHostState: SnackbarHostState, state: WelcomeUiState) {
    val copiedAccountNumberMessage = stringResource(id = R.string.copied_mullvad_account_number)
    val copyToClipboard =
        createCopyToClipboardHandle(snackbarHostState = snackbarHostState, isSensitive = true)
    val onCopyToClipboard = {
        copyToClipboard(state.accountNumber?.value ?: "", copiedAccountNumberMessage)
    }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween,
        modifier =
            Modifier.fillMaxWidth()
                .clickable(onClick = onCopyToClipboard)
                .padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    top = Dimens.cellVerticalSpacing,
                    bottom = Dimens.mediumPadding,
                ),
    ) {
        Text(
            text = state.accountNumber?.value?.groupWithSpaces() ?: "",
            modifier = Modifier.weight(1f),
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
            style = MaterialTheme.typography.labelLarge,
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
    verificationPending: Boolean,
    onAddMoreTimeClick: () -> Unit,
    onDisconnectClick: () -> Unit,
    onInfoClick: () -> Unit,
) {
    Column(
        modifier =
            Modifier.fillMaxWidth()
                .padding(
                    top = Dimens.mediumPadding,
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                )
    ) {
        Spacer(modifier = Modifier.padding(top = Dimens.screenTopMargin))
        if (showDisconnectButton) {
            NegativeButton(
                onClick = onDisconnectClick,
                text = stringResource(id = R.string.disconnect),
                modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
            )
        }
        if (verificationPending) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                IconButton(
                    onClick = onInfoClick,
                    modifier = Modifier.testTag(PLAY_PAYMENT_INFO_ICON_TEST_TAG),
                ) {
                    Icon(
                        imageVector = Icons.Default.Info,
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
        VariantButton(
            onClick = onAddMoreTimeClick,
            text = stringResource(id = R.string.add_time),
            modifier = Modifier.padding(bottom = Dimens.buttonSpacing),
        )
    }
}
