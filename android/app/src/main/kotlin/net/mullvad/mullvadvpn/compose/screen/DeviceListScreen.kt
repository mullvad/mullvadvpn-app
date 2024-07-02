package net.mullvad.mullvadvpn.compose.screen

import android.util.Log
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.LoginDestination
import com.ramcosta.composedestinations.generated.destinations.RemoveDeviceConfirmationDialogDestination
import com.ramcosta.composedestinations.generated.destinations.SettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.DeviceListPreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.DeviceItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.compose.transitions.DefaultTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.AccountNumber
import net.mullvad.mullvadvpn.lib.model.Device
import net.mullvad.mullvadvpn.lib.model.DeviceId
import net.mullvad.mullvadvpn.lib.model.GetDeviceListError
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText
import net.mullvad.mullvadvpn.util.formatDate
import net.mullvad.mullvadvpn.viewmodel.DeviceListSideEffect
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Composable
@Preview
private fun PreviewDeviceListScreenContent(
    @PreviewParameter(DeviceListPreviewParameterProvider::class) devices: List<DeviceItemUiState>
) {
    AppTheme { DeviceListScreen(state = DeviceListUiState.Content(devices = devices)) }
}

@Composable
@Preview
private fun PreviewDeviceListScreenEmpty() {
    AppTheme { DeviceListScreen(state = DeviceListUiState.Content(devices = emptyList())) }
}

@Composable
@Preview
private fun PreviewDeviceListLoading() {
    AppTheme { DeviceListScreen(state = DeviceListUiState.Loading) }
}

@Composable
@Preview
private fun PreviewDeviceListError() {
    AppTheme {
        DeviceListScreen(
            state =
                DeviceListUiState.Error(GetDeviceListError.Unknown(IllegalStateException("Error")))
        )
    }
}

@Destination<RootGraph>(style = DefaultTransition::class)
@Composable
fun DeviceList(
    navigator: DestinationsNavigator,
    accountNumber: String,
    confirmRemoveResultRecipient:
        ResultRecipient<RemoveDeviceConfirmationDialogDestination, DeviceId>
) {
    val viewModel =
        koinViewModel<DeviceListViewModel>(
            parameters = { parametersOf(AccountNumber(accountNumber)) }
        )
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
        minActiveState = Lifecycle.State.RESUMED
    ) { sideEffect ->
        when (sideEffect) {
            DeviceListSideEffect.FailedToRemoveDevice -> {
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.failed_to_remove_device)
                    )
                }
            }
        }
    }

    DeviceListScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onContinueWithLogin =
            dropUnlessResumed {
                navigator.navigate(LoginDestination(accountNumber)) {
                    launchSingleTop = true
                    popUpTo(LoginDestination) { inclusive = true }
                }
            },
        onSettingsClicked = dropUnlessResumed { navigator.navigate(SettingsDestination) },
        onTryAgainClicked = viewModel::fetchDevices,
        navigateToRemoveDeviceConfirmationDialog =
            dropUnlessResumed<Device> {
                // Logger.i("DeviceList: remove device: $it")
                Log.d("mullvad", "DeviceList: remove device: $it")
                navigator.navigate(RemoveDeviceConfirmationDialogDestination(it))
            }
    )
}

@Composable
fun DeviceListScreen(
    state: DeviceListUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onBackClick: () -> Unit = {},
    onContinueWithLogin: () -> Unit = {},
    onSettingsClicked: () -> Unit = {},
    onTryAgainClicked: () -> Unit = {},
    navigateToRemoveDeviceConfirmationDialog: (device: Device) -> Unit = {}
) {

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClicked,
        onAccountClicked = null,
        snackbarHostState = snackbarHostState
    ) {
        Column(
            modifier = Modifier.fillMaxSize().padding(it),
        ) {
            val scrollState = rememberScrollState()
            Column(
                modifier =
                    Modifier.drawVerticalScrollbar(
                            scrollState,
                            MaterialTheme.colorScheme.onBackground
                        )
                        .verticalScroll(scrollState)
                        .weight(1f)
                        .fillMaxWidth(),
            ) {
                DeviceListHeader(state)
                when (state) {
                    is DeviceListUiState.Content ->
                        DeviceListContent(
                            state,
                            navigateToRemoveDeviceConfirmationDialog =
                                navigateToRemoveDeviceConfirmationDialog
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
            modifier = Modifier.padding(Dimens.smallPadding).align(Alignment.CenterHorizontally)
        )
        PrimaryButton(
            onClick = tryAgain,
            text = stringResource(id = R.string.try_again),
            modifier =
                Modifier.padding(
                    top = Dimens.buttonSpacing,
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin
                )
        )
    }
}

@Composable
private fun ColumnScope.DeviceListContent(
    state: DeviceListUiState.Content,
    navigateToRemoveDeviceConfirmationDialog: (Device) -> Unit
) {
    state.devices.forEachIndexed { index, (device, loading) ->
        DeviceListItem(
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
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(top = Dimens.iconFailSuccessTopMargin)
                        .size(Dimens.bigIconSize)
            )
        is DeviceListUiState.Error ->
            Image(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null, // No meaningful user info or action.
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(top = Dimens.iconFailSuccessTopMargin)
                        .size(Dimens.bigIconSize)
            )
        DeviceListUiState.Loading ->
            MullvadCircularProgressIndicatorLarge(
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(top = Dimens.iconFailSuccessTopMargin)
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
        color = MaterialTheme.colorScheme.onBackground,
        modifier =
            Modifier.padding(
                start = Dimens.sideMargin,
                end = Dimens.sideMargin,
                top = Dimens.screenVerticalMargin
            ),
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
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onBackground,
            modifier =
                Modifier.wrapContentHeight()
                    .animateContentSize()
                    .padding(
                        top = Dimens.smallPadding,
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.spacingAboveButton
                    )
        )
    }
}

@Composable
private fun DeviceListItem(device: Device, isLoading: Boolean, onDeviceRemovalClicked: () -> Unit) {
    BaseCell(
        isRowEnabled = false,
        headlineContent = {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = device.displayName(),
                    style = MaterialTheme.typography.listItemText,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text =
                        stringResource(id = R.string.created_x, device.creationDate.formatDate()),
                    style = MaterialTheme.typography.listItemSubText,
                    color =
                        MaterialTheme.colorScheme.onPrimary
                            .copy(alpha = AlphaDescription)
                            .compositeOver(MaterialTheme.colorScheme.primary)
                )
            }
        },
        bodyView = {
            if (isLoading) {
                MullvadCircularProgressIndicatorMedium(
                    modifier = Modifier.padding(Dimens.smallPadding)
                )
            } else {
                IconButton(onClick = onDeviceRemovalClicked) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_close),
                        contentDescription = stringResource(id = R.string.remove_button),
                        tint = MaterialTheme.colorScheme.onPrimary,
                        modifier = Modifier.size(size = Dimens.deleteIconSize)
                    )
                }
            }
        },
        endPadding = Dimens.smallPadding,
    )
}

@Composable
private fun DeviceListButtonPanel(
    state: DeviceListUiState,
    onContinueWithLogin: () -> Unit,
    onBackClick: () -> Unit
) {
    Column(
        modifier =
            Modifier.padding(
                start = Dimens.sideMargin,
                end = Dimens.sideMargin,
                top = Dimens.spacingAboveButton,
                bottom = Dimens.screenVerticalMargin
            )
    ) {
        VariantButton(
            text = stringResource(id = R.string.continue_login),
            onClick = onContinueWithLogin,
            isEnabled = state is DeviceListUiState.Content && !state.hasTooManyDevices,
            background = MaterialTheme.colorScheme.secondary
        )

        PrimaryButton(
            text = stringResource(id = R.string.back),
            onClick = onBackClick,
            modifier = Modifier.padding(top = Dimens.buttonSpacing)
        )
    }
}
