package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.ShowDeviceRemovalDialog
import net.mullvad.mullvadvpn.compose.state.DeviceListItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.common.util.parseAsDateTime
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemSubText
import net.mullvad.mullvadvpn.lib.theme.typeface.listItemText
import net.mullvad.mullvadvpn.model.Device
import net.mullvad.mullvadvpn.util.formatDate

@Composable
@Preview
private fun PreviewDeviceListScreen() {
    AppTheme {
        DeviceListScreen(
            state =
                DeviceListUiState(
                    deviceUiItems =
                        listOf(
                            DeviceListItemUiState(
                                device =
                                    Device(
                                        id = "ID",
                                        name = "Powerful Hen",
                                        pubkey = ByteArray(10),
                                        created = "2022-12-17 17:12:00 PST"
                                    ),
                                isLoading = false
                            ),
                            DeviceListItemUiState(
                                device =
                                    Device(
                                        id = "ID",
                                        name = "Weak Ostrich",
                                        pubkey = ByteArray(10),
                                        created = "2021-02-06 19:21:41 PST"
                                    ),
                                isLoading = false
                            )
                        ),
                    isLoading = true,
                    stagedDevice = null
                )
        )
    }
}

@Composable
fun DeviceListScreen(
    state: DeviceListUiState,
    onBackClick: () -> Unit = {},
    onContinueWithLogin: () -> Unit = {},
    onSettingsClicked: () -> Unit = {},
    onDeviceRemovalClicked: (deviceId: String) -> Unit = {},
    onDismissDeviceRemovalDialog: () -> Unit = {},
    onConfirmDeviceRemovalDialog: () -> Unit = {}
) {
    if (state.stagedDevice != null) {
        ShowDeviceRemovalDialog(
            onDismiss = onDismissDeviceRemovalDialog,
            onConfirmDeviceRemovalDialog,
            device = state.stagedDevice
        )
    }

    ScaffoldWithTopBar(
        topBarColor = MaterialTheme.colorScheme.primary,
        statusBarColor = MaterialTheme.colorScheme.primary,
        navigationBarColor = MaterialTheme.colorScheme.background,
        iconTintColor = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClicked,
        onAccountClicked = null,
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
            ) {
                DeviceListHeader(state = state)

                state.deviceUiItems.forEachIndexed { index, deviceUiState ->
                    DeviceListItem(
                        deviceUiState = deviceUiState,
                    ) {
                        onDeviceRemovalClicked(deviceUiState.device.id)
                    }
                    if (state.deviceUiItems.lastIndex != index) {
                        Divider()
                    }
                }
            }
            DeviceListButtonPanel(state, onContinueWithLogin, onBackClick)
        }
    }
}

@Composable
private fun ColumnScope.DeviceListHeader(state: DeviceListUiState) {
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

    Text(
        text =
            stringResource(
                id =
                    if (state.hasTooManyDevices) {
                        R.string.max_devices_warning_title
                    } else {
                        R.string.max_devices_resolved_title
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

@Composable
private fun DeviceListItem(
    deviceUiState: DeviceListItemUiState,
    onDeviceRemovalClicked: () -> Unit
) {
    BaseCell(
        isRowEnabled = false,
        title = {
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text = deviceUiState.device.displayName(),
                    style = MaterialTheme.typography.listItemText,
                    color = MaterialTheme.colorScheme.onPrimary
                )
                Text(
                    modifier = Modifier.fillMaxWidth(),
                    text =
                        deviceUiState.device.created.parseAsDateTime()?.let { creationDate ->
                            stringResource(id = R.string.created_x, creationDate.formatDate())
                        }
                            ?: "",
                    style = MaterialTheme.typography.listItemSubText,
                    color =
                        MaterialTheme.colorScheme.onPrimary
                            .copy(alpha = AlphaDescription)
                            .compositeOver(MaterialTheme.colorScheme.primary)
                )
            }
        },
        bodyView = {
            if (deviceUiState.isLoading) {
                CircularProgressIndicator(
                    strokeWidth = Dimens.loadingSpinnerStrokeWidth,
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier =
                        Modifier.height(Dimens.loadingSpinnerSize).width(Dimens.loadingSpinnerSize)
                )
            } else {
                IconButton(onClick = onDeviceRemovalClicked) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_close),
                        contentDescription = "Remove",
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
            isEnabled = state.hasTooManyDevices.not() && state.isLoading.not(),
            background = MaterialTheme.colorScheme.secondary
        )

        PrimaryButton(
            text = stringResource(id = R.string.back),
            onClick = onBackClick,
            modifier = Modifier.padding(top = Dimens.mediumPadding)
        )
    }
}
