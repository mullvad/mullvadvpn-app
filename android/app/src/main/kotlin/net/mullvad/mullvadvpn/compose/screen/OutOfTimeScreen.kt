package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.platform.LocalResources
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
import com.ramcosta.composedestinations.generated.destinations.RedeemVoucherDestination
import com.ramcosta.composedestinations.generated.destinations.SettingsDestination
import com.ramcosta.composedestinations.generated.destinations.VerificationPendingDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.NegativeButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.AddTimeBottomSheet
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBarAndDeviceName
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.createOpenAccountPageHook
import net.mullvad.mullvadvpn.compose.preview.OutOfTimeScreenPreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.OutOfTimeUiState
import net.mullvad.mullvadvpn.compose.transitions.HomeTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.tag.OUT_OF_TIME_SCREEN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.PLAY_PAYMENT_INFO_ICON_TEST_TAG
import net.mullvad.mullvadvpn.viewmodel.OutOfTimeViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Disconnected|Connecting|Error")
@Composable
private fun PreviewOutOfTimeScreen(
    @PreviewParameter(OutOfTimeScreenPreviewParameterProvider::class) state: OutOfTimeUiState
) {
    AppTheme {
        OutOfTimeScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onDisconnectClick = {},
            onSettingsClick = {},
            onAccountClick = {},
            onRedeemVoucherClick = {},
            onPlayPaymentInfoClick = {},
        )
    }
}

@Destination<RootGraph>(style = HomeTransition::class)
@Composable
fun OutOfTime(navigator: DestinationsNavigator) {
    val vm = koinViewModel<OutOfTimeViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
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
                    message = resources.getString(R.string.error_occurred)
                )
        }
    }

    OutOfTimeScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onAccountClick = dropUnlessResumed { navigator.navigate(AccountDestination) },
        onRedeemVoucherClick = dropUnlessResumed { navigator.navigate(RedeemVoucherDestination) },
        onPlayPaymentInfoClick =
            dropUnlessResumed { navigator.navigate(VerificationPendingDestination) },
        onDisconnectClick = vm::onDisconnectClick,
    )
}

@Composable
fun OutOfTimeScreen(
    state: OutOfTimeUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onDisconnectClick: () -> Unit,
    onSettingsClick: () -> Unit,
    onAccountClick: () -> Unit,
    onRedeemVoucherClick: () -> Unit,
    onPlayPaymentInfoClick: () -> Unit,
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
                    .padding(it)
                    .padding(
                        top = Dimens.screenTopMargin,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenBottomMargin,
                    )
                    .verticalScroll(scrollState)
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .background(color = MaterialTheme.colorScheme.surface)
        ) {
            Content(showSitePayment = state.showSitePayment)
            Spacer(modifier = Modifier.weight(1f).defaultMinSize(minHeight = Dimens.verticalSpace))
            // Button area
            ButtonPanel(
                state = state,
                onDisconnectClick = onDisconnectClick,
                onAddMoreTimeClick = { addTimeBottomSheetState = true },
                onInfoClick = onPlayPaymentInfoClick,
            )
        }
    }
}

@Composable
private fun ColumnScope.Content(showSitePayment: Boolean) {
    Image(
        painter = painterResource(id = R.drawable.icon_fail),
        contentDescription = null,
        modifier =
            Modifier.align(Alignment.CenterHorizontally).padding(bottom = Dimens.mediumSpacer),
    )
    Text(
        text = stringResource(id = R.string.out_of_time),
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.testTag(OUT_OF_TIME_SCREEN_TITLE_TEST_TAG),
    )
    Text(
        text =
            buildString {
                append(stringResource(R.string.account_credit_has_expired))
                if (showSitePayment) {
                    append(" ")
                    append(stringResource(R.string.add_time_to_account))
                }
            },
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.padding(top = Dimens.mediumPadding),
    )
}

@Composable
private fun ButtonPanel(
    state: OutOfTimeUiState,
    onDisconnectClick: () -> Unit,
    onAddMoreTimeClick: () -> Unit,
    onInfoClick: () -> Unit,
) {
    Column(verticalArrangement = Arrangement.spacedBy(Dimens.buttonSpacing)) {
        if (state.tunnelState.isSecured()) {
            NegativeButton(
                onClick = onDisconnectClick,
                text = stringResource(id = R.string.disconnect),
            )
        }
        if (state.verificationPending) {
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
        VariantButton(onClick = onAddMoreTimeClick, text = stringResource(id = R.string.add_time))
    }
}
