package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ManageDevicesRemoveConfirmationDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.BaseSubtitleCell
import net.mullvad.mullvadvpn.compose.component.DeviceListItem
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.ManageDevicesUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.compose.transitions.DefaultTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.viewmodel.ManageDevicesSideEffect
import net.mullvad.mullvadvpn.viewmodel.ManageDevicesViewModel
import org.koin.androidx.compose.koinViewModel

@Composable
@Preview("Normal|TooMany|Empty|Loading|Error")
private fun PreviewDeviceListScreenContent(
    @PreviewParameter(ManageDevicesUiStatePreviewParameterProvider::class)
    state: Lce<Unit, ManageDevicesUiState, GetDeviceListError>
) {
    AppTheme { ManageDevicesScreen(state = state, SnackbarHostState(), {}, {}, {}) }
}

private typealias StateLce = Lce<Unit, ManageDevicesUiState, GetDeviceListError>

@Destination<RootGraph>(style = DefaultTransition::class, navArgs = DeviceListNavArgs::class)
@Composable
fun ManageDevices(
    navigator: DestinationsNavigator,
    confirmRemoveResultRecipient:
        ResultRecipient<ManageDevicesRemoveConfirmationDestination, DeviceId>,
) {
    val viewModel = koinViewModel<ManageDevicesViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    confirmRemoveResultRecipient.OnNavResultValue { deviceId ->
        viewModel.removeDevice(deviceIdToRemove = deviceId)
    }

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    CollectSideEffectWithLifecycle(
        viewModel.uiSideEffect,
        minActiveState = Lifecycle.State.RESUMED,
    ) { sideEffect ->
        when (sideEffect) {
            ManageDevicesSideEffect.FailedToRemoveDevice -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.failed_to_remove_device)
                    )
                }
            }
        }
    }

    ManageDevicesScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onTryAgainClicked = viewModel::fetchDevices,
        navigateToRemoveDeviceConfirmationDialog =
            dropUnlessResumed<Device> {
                navigator.navigate(ManageDevicesRemoveConfirmationDestination(it))
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
    Column(modifier) {
        BaseSubtitleCell(
            text = stringResource(R.string.manage_devices_description),
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
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
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
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
        DeviceListItem(device = device, isLoading = loading, isCurrentDevice = isCurrentDevice) {
            navigateToRemoveDeviceConfirmationDialog(device)
        }
        if (state.devices.lastIndex != index) {
            HorizontalDivider()
        }
    }
}
