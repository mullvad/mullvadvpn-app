package net.mullvad.mullvadvpn.feature.login.impl.devicelist

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.login.api.LoginNavKey
import net.mullvad.mullvadvpn.feature.login.api.RemoveDeviceConfirmationDialogResult
import net.mullvad.mullvadvpn.feature.login.api.RemoveDeviceNavKey
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.DeviceListItem
import net.mullvad.mullvadvpn.lib.ui.component.positionForIndex
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview("Normal|TooMany|Empty|Loading|Error")
private fun PreviewDeviceListScreenContent(
    @PreviewParameter(DeviceListUiStatePreviewParameterProvider::class) state: DeviceListUiState
) {
    AppTheme {
        DeviceListScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onBackClick = {},
            onContinueWithLogin = {},
            onSettingsClicked = {},
            onTryAgainClicked = {},
            navigateToRemoveDeviceConfirmationDialog = {},
        )
    }
}

@Composable
fun DeviceList(navigator: Navigator, accountNumber: AccountNumber) {
    val viewModel = koinViewModel<DeviceListViewModel> { parametersOf(accountNumber) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LocalResultStore.current.consumeResult<RemoveDeviceConfirmationDialogResult> { result ->
        viewModel.removeDevice(deviceIdToRemove = result.device)
    }

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(
        viewModel.uiSideEffect,
        minActiveState = Lifecycle.State.RESUMED,
    ) { sideEffect ->
        when (sideEffect) {
            DeviceListSideEffect.FailedToRemoveDevice -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.failed_to_remove_device)
                    )
                }
            }
            is DeviceListSideEffect.NavigateToLogin ->
                navigator.navigate(
                    LoginNavKey(sideEffect.accountNumber.value),
                    clearBackStack = true,
                )
        }
    }

    DeviceListScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onContinueWithLogin = viewModel::continueToLogin,
        onSettingsClicked = dropUnlessResumed { navigator.navigate(SettingsNavKey) },
        onTryAgainClicked = viewModel::fetchDevices,
        navigateToRemoveDeviceConfirmationDialog =
            dropUnlessResumed<Device> { navigator.navigate(RemoveDeviceNavKey(it)) },
    )
}

@Composable
fun DeviceListScreen(
    state: DeviceListUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onBackClick: () -> Unit,
    onContinueWithLogin: () -> Unit,
    onSettingsClicked: () -> Unit,
    onTryAgainClicked: () -> Unit,
    navigateToRemoveDeviceConfirmationDialog: (device: Device) -> Unit,
) {

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary,
        onSettingsClicked = onSettingsClicked,
        onAccountClicked = null,
        snackbarHostState = snackbarHostState,
    ) {
        Column(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .padding(
                        bottom = Dimens.screenBottomMargin,
                        start = Dimens.sideMarginNew,
                        end = Dimens.sideMarginNew,
                    )
        ) {
            val scrollState = rememberScrollState()
            Column(
                modifier =
                    Modifier.drawVerticalScrollbar(scrollState, MaterialTheme.colorScheme.onSurface)
                        .padding(top = Dimens.screenTopMargin)
                        .verticalScroll(scrollState)
                        .weight(1f)
                        .fillMaxWidth()
            ) {
                DeviceListHeader(state)
                when (state) {
                    is DeviceListUiState.Content ->
                        DeviceListContent(
                            state,
                            navigateToRemoveDeviceConfirmationDialog =
                                navigateToRemoveDeviceConfirmationDialog,
                        )
                    is DeviceListUiState.Error -> DeviceListError(onTryAgainClicked)
                    DeviceListUiState.Loading -> {}
                }
            }
            DeviceListButtonPanel(state, onContinueWithLogin, onBackClick)
        }
    }
}

@Composable
private fun ColumnScope.DeviceListError(tryAgain: () -> Unit) {
    Column(Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
        Text(
            text = stringResource(id = R.string.failed_to_fetch_devices),
            modifier = Modifier.padding(Dimens.smallPadding).align(Alignment.CenterHorizontally),
            style = MaterialTheme.typography.bodyMedium,
        )
        PrimaryButton(
            onClick = tryAgain,
            text = stringResource(id = R.string.try_again),
            modifier = Modifier.padding(top = Dimens.buttonSpacing),
        )
    }
}

@Composable
private fun DeviceListContent(
    state: DeviceListUiState.Content,
    navigateToRemoveDeviceConfirmationDialog: (Device) -> Unit,
) {
    state.devices.forEachIndexed { index, (device, loading) ->
        DeviceListItem(
            position = state.devices.positionForIndex(index),
            device = device,
            isLoading = loading,
        ) {
            navigateToRemoveDeviceConfirmationDialog(device)
        }
        if (state.devices.lastIndex != index) {
            HorizontalDivider()
        }
    }
}

@Composable
private fun ColumnScope.DeviceListHeader(state: DeviceListUiState) {
    when (state) {
        is DeviceListUiState.Content ->
            Image(
                painter =
                    painterResource(
                        id =
                            if (state.hasTooManyDevices) {
                                R.drawable.icon_fail
                            } else {
                                R.drawable.icon_success
                            }
                    ),
                contentDescription = null, // No meaningful user info or action.
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )
        is DeviceListUiState.Error ->
            Image(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null, // No meaningful user info or action.
                modifier = Modifier.align(Alignment.CenterHorizontally),
            )
        DeviceListUiState.Loading ->
            MullvadCircularProgressIndicatorLarge(
                modifier = Modifier.align(Alignment.CenterHorizontally)
            )
    }

    Text(
        text =
            stringResource(
                id =
                    if (state is DeviceListUiState.Content && !state.hasTooManyDevices) {
                        R.string.max_devices_resolved_title
                    } else {
                        R.string.max_devices_warning_title
                    }
            ),
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.padding(top = Dimens.screenTopMargin),
    )

    if (state is DeviceListUiState.Content) {
        Text(
            text =
                stringResource(
                    id =
                        if (state.hasTooManyDevices) {
                            R.string.max_devices_warning_description
                        } else {
                            R.string.max_devices_resolved_description
                        }
                ),
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
            modifier =
                Modifier.wrapContentHeight()
                    .animateContentSize()
                    .padding(top = Dimens.smallPadding, bottom = Dimens.cellVerticalSpacing),
        )
    }
}

@Composable
private fun DeviceListButtonPanel(
    state: DeviceListUiState,
    onContinueWithLogin: () -> Unit,
    onBackClick: () -> Unit,
) {
    Column(modifier = Modifier.padding(top = Dimens.mediumPadding)) {
        VariantButton(
            text = stringResource(id = R.string.continue_login),
            onClick = onContinueWithLogin,
            isEnabled = state is DeviceListUiState.Content && !state.hasTooManyDevices,
            background = MaterialTheme.colorScheme.positive,
        )

        PrimaryButton(
            text = stringResource(id = R.string.back),
            onClick = onBackClick,
            modifier = Modifier.padding(top = Dimens.buttonSpacing),
        )
    }
}
