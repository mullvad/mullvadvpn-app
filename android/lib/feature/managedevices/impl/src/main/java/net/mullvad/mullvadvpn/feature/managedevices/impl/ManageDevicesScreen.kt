package net.mullvad.mullvadvpn.feature.managedevices.impl

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import net.mullvad.mullvadvpn.feature.managedevices.api.ManageDevicesRemoveConfirmationNavKey
import net.mullvad.mullvadvpn.feature.managedevices.api.ManageDevicesRemoveConfirmationNavResult
import net.mullvad.mullvadvpn.lib.common.Lce
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.DeviceListItem
import net.mullvad.mullvadvpn.lib.ui.component.positionForIndex
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview("Normal|TooMany|Empty|Loading|Error")
private fun PreviewDeviceListScreenContent(
    @PreviewParameter(ManageDevicesUiStatePreviewParameterProvider::class)
    state: Lce<Unit, ManageDevicesUiState, GetDeviceListError>
) {
    AppTheme {
        ManageDevicesScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onBackClick = {},
            onTryAgainClicked = {},
            navigateToRemoveDeviceConfirmationDialog = {},
        )
    }
}

private typealias StateLce = Lce<Unit, ManageDevicesUiState, GetDeviceListError>

@Composable
fun ManageDevices(accountNumber: AccountNumber, navigator: Navigator) {
    val viewModel = koinViewModel<ManageDevicesViewModel> { parametersOf(accountNumber) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LocalResultStore.current.consumeResult<ManageDevicesRemoveConfirmationNavResult> {
        viewModel.removeDevice(deviceIdToRemove = it.deviceId)
    }

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(
        viewModel.uiSideEffect,
        minActiveState = Lifecycle.State.RESUMED,
    ) { sideEffect ->
        when (sideEffect) {
            ManageDevicesSideEffect.FailedToRemoveDevice -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.failed_to_remove_device)
                    )
                }
            }
        }
    }

    ManageDevicesScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onTryAgainClicked = viewModel::fetchDevices,
        navigateToRemoveDeviceConfirmationDialog =
            dropUnlessResumed<Device> {
                navigator.navigate(ManageDevicesRemoveConfirmationNavKey(it))
            },
    )
}

@Composable
fun ManageDevicesScreen(
    state: StateLce,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onBackClick: () -> Unit,
    onTryAgainClicked: () -> Unit,
    navigateToRemoveDeviceConfirmationDialog: (device: Device) -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.manage_devices),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        when (state) {
            is Lce.Content ->
                Content(modifier, state.value, navigateToRemoveDeviceConfirmationDialog)
            is Lce.Error -> Error(modifier, onTryAgainClicked)
            is Lce.Loading -> Loading(modifier)
        }
    }
}

@Composable
private fun Content(
    modifier: Modifier,
    state: ManageDevicesUiState,
    navigateToRemoveDeviceConfirmationDialog: (device: Device) -> Unit,
) {
    Column(modifier.padding(horizontal = Dimens.sideMarginNew)) {
        Text(
            text = stringResource(id = R.string.manage_devices_description),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(Dimens.verticalSpace))
        ManageDevicesItems(
            state = state,
            navigateToRemoveDeviceConfirmationDialog = navigateToRemoveDeviceConfirmationDialog,
        )
    }
}

@Composable
private fun Error(modifier: Modifier, tryAgain: () -> Unit) {
    Column(modifier, verticalArrangement = Arrangement.Center) {
        Text(
            text = stringResource(id = R.string.failed_to_fetch_devices),
            modifier = Modifier.padding(Dimens.smallPadding).align(Alignment.CenterHorizontally),
        )
        PrimaryButton(
            onClick = tryAgain,
            text = stringResource(id = R.string.try_again),
            modifier =
                Modifier.padding(
                    top = Dimens.buttonSpacing,
                    start = Dimens.sideMarginNew,
                    end = Dimens.sideMarginNew,
                ),
        )
    }
}

@Composable
private fun Loading(modifier: Modifier) {
    Box(modifier, contentAlignment = Alignment.Center) {
        MullvadCircularProgressIndicatorMedium(modifier = Modifier.padding(Dimens.smallPadding))
    }
}

@Composable
private fun ManageDevicesItems(
    state: ManageDevicesUiState,
    navigateToRemoveDeviceConfirmationDialog: (Device) -> Unit,
) {
    state.devices.forEachIndexed { index, (device, loading, isCurrentDevice) ->
        DeviceListItem(
            position = state.devices.positionForIndex(index),
            device = device,
            isLoading = loading,
            isCurrentDevice = isCurrentDevice,
        ) {
            navigateToRemoveDeviceConfirmationDialog(device)
        }
        if (state.devices.lastIndex != index) {
            HorizontalDivider()
        }
    }
}
