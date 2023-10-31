package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.ListItem
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.dialog.ShowDeviceRemovalDialog
import net.mullvad.mullvadvpn.compose.state.DeviceListItemUiState
import net.mullvad.mullvadvpn.compose.state.DeviceListUiState
import net.mullvad.mullvadvpn.lib.common.util.parseAsDateTime
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar
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
                                        name = "Name",
                                        pubkey = ByteArray(10),
                                        created = "2002-12-12"
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
        ConstraintLayout(
            modifier =
                Modifier.fillMaxHeight()
                    .fillMaxWidth()
                    .padding(it)
                    .background(MaterialTheme.colorScheme.background)
        ) {
            val (content, buttons) = createRefs()

            Column(
                modifier =
                    Modifier.constrainAs(content) {
                            top.linkTo(parent.top)
                            bottom.linkTo(buttons.top)
                            height = Dimension.fillToConstraints
                            width = Dimension.matchParent
                        }
                        .verticalScroll(rememberScrollState())
            ) {
                ConstraintLayout(modifier = Modifier.fillMaxWidth().wrapContentHeight()) {
                    val (icon, message, list) = createRefs()

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
                            Modifier.constrainAs(icon) {
                                    top.linkTo(parent.top)
                                    start.linkTo(parent.start)
                                    end.linkTo(parent.end)
                                }
                                .padding(top = Dimens.iconFailSuccessTopMargin)
                                .size(Dimens.bigIconSize)
                    )

                    Column(
                        modifier =
                            Modifier.constrainAs(message) {
                                    top.linkTo(icon.bottom)
                                    start.linkTo(parent.start)
                                    end.linkTo(parent.end)
                                    width = Dimension.fillToConstraints
                                }
                                .padding(
                                    start = Dimens.sideMargin,
                                    end = Dimens.sideMargin,
                                    top = Dimens.screenVerticalMargin
                                ),
                    ) {
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
                            color = MaterialTheme.colorScheme.onBackground
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
                                    .padding(top = Dimens.smallPadding)
                        )
                    }

                    Box(
                        modifier =
                            Modifier.constrainAs(list) {
                                    top.linkTo(message.bottom)
                                    height = Dimension.wrapContent
                                    width = Dimension.matchParent
                                }
                                .padding(top = Dimens.spacingAboveButton)
                    ) {
                        Column {
                            state.deviceUiItems.forEach { deviceUiState ->
                                ListItem(
                                    text = deviceUiState.device.displayName(),
                                    subText =
                                        deviceUiState.device.created.parseAsDateTime()?.let {
                                            creationDate ->
                                            stringResource(
                                                id = R.string.created_x,
                                                creationDate.formatDate()
                                            )
                                        },
                                    height = Dimens.listItemHeightExtra,
                                    isLoading = deviceUiState.isLoading,
                                    iconResourceId = R.drawable.icon_close
                                ) {
                                    onDeviceRemovalClicked(deviceUiState.device.id)
                                }
                            }
                        }
                    }
                }
            }

            Column(
                modifier =
                    Modifier.constrainAs(buttons) {
                            bottom.linkTo(parent.bottom)
                            start.linkTo(parent.start)
                            end.linkTo(parent.end)
                            width = Dimension.fillToConstraints
                        }
                        .padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
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
                    modifier = Modifier.padding(top = Dimens.buttonSeparation)
                )
            }
        }
    }
}
