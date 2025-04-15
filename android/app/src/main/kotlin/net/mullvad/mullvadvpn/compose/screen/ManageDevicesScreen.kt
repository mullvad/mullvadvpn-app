package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Clear
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ManageDevicesRemoveConfirmationDialogDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.cell.BaseSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.ManageDevicesUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.ManageDevicesUiState
import net.mullvad.mullvadvpn.compose.transitions.DefaultTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.formatDate
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText
import net.mullvad.mullvadvpn.util.Lce
import net.mullvad.mullvadvpn.viewmodel.ManageDevicesSideEffect
import net.mullvad.mullvadvpn.viewmodel.ManageDevicesViewModel
import org.koin.androidx.compose.koinViewModel

@Composable
@Preview("Normal|TooMany|Empty|Loading|Error")
private fun PreviewDeviceListScreenContent(
    @PreviewParameter(ManageDevicesUiStatePreviewParameterProvider::class)
    state: Lce<ManageDevicesUiState, GetDeviceListError>
) {
    AppTheme { ManageDevicesScreen(state = state, SnackbarHostState(), {}, {}, {}) }
}

private typealias StateLce = Lce<ManageDevicesUiState, GetDeviceListError>

@Destination<RootGraph>(style = DefaultTransition::class, navArgs = DeviceListNavArgs::class)
@Composable
fun ManageDevices(
    navigator: DestinationsNavigator,
    confirmRemoveResultRecipient:
        ResultRecipient<ManageDevicesRemoveConfirmationDialogDestination, DeviceId>,
) {
    val viewModel = koinViewModel<ManageDevicesViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    confirmRemoveResultRecipient.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                viewModel.removeDevice(deviceIdToRemove = it.value)
            }
        }
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
                navigator.navigate(ManageDevicesRemoveConfirmationDialogDestination(it))
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
        Column(
            modifier = modifier,
            verticalArrangement = if (state.isLoading()) Arrangement.Center else Arrangement.Top,
        ) {
            when (state) {
                is Lce.Content -> {
                    BaseSubtitleCell(
                        text = stringResource(R.string.manage_devices_description),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    ManageDevicesContent(
                        state.value,
                        navigateToRemoveDeviceConfirmationDialog =
                            navigateToRemoveDeviceConfirmationDialog,
                    )
                }
                is Lce.Error -> ManageDevicesError(onTryAgainClicked)
                Lce.Loading ->
                    MullvadCircularProgressIndicatorMedium(
                        modifier =
                            Modifier.padding(Dimens.smallPadding)
                                .align(Alignment.CenterHorizontally)
                    )
            }
        }
    }
}

@Composable
private fun ColumnScope.ManageDevicesError(tryAgain: () -> Unit) {
    Column(Modifier.weight(1f), verticalArrangement = Arrangement.Center) {
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
private fun ColumnScope.ManageDevicesContent(
    state: ManageDevicesUiState,
    navigateToRemoveDeviceConfirmationDialog: (Device) -> Unit,
) {
    state.devices.forEachIndexed { index, (device, loading, isCurrentDevice) ->
        ManageDevicesItem(device = device, isLoading = loading, isCurrentDevice = isCurrentDevice) {
            navigateToRemoveDeviceConfirmationDialog(device)
        }
        if (state.devices.lastIndex != index) {
            HorizontalDivider()
        }
    }
}

@Composable
private fun ManageDevicesItem(
    device: Device,
    isLoading: Boolean,
    isCurrentDevice: Boolean,
    onDeviceRemovalClicked: () -> Unit,
) {
    TwoRowCell(
        titleStyle = MaterialTheme.typography.listItemText,
        titleColor = MaterialTheme.colorScheme.onPrimary,
        subtitleStyle = MaterialTheme.typography.listItemSubText,
        subtitleColor = MaterialTheme.colorScheme.onSurfaceVariant,
        titleText = device.displayName(),
        subtitleText = stringResource(id = R.string.created_x, device.creationDate.formatDate()),
        bodyView = {
            if (isLoading) {
                MullvadCircularProgressIndicatorMedium(
                    modifier = Modifier.padding(Dimens.smallPadding)
                )
            } else if (isCurrentDevice) {
                Text(
                    modifier = Modifier.padding(Dimens.smallPadding),
                    text = "Current device",
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    style = MaterialTheme.typography.labelLarge,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            } else {
                IconButton(onClick = onDeviceRemovalClicked) {
                    Icon(
                        imageVector = Icons.Default.Clear,
                        contentDescription = stringResource(id = R.string.remove_button),
                        tint = MaterialTheme.colorScheme.onPrimary,
                        modifier = Modifier.size(size = Dimens.deleteIconSize),
                    )
                }
            }
        },
        onCellClicked = null,
        endPadding = Dimens.smallPadding,
        minHeight = Dimens.cellHeight,
    )
}
