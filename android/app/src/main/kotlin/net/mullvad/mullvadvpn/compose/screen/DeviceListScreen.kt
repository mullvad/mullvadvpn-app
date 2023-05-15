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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ActionButton
import net.mullvad.mullvadvpn.compose.component.ListItem
import net.mullvad.mullvadvpn.compose.dialog.ShowDeviceRemovalDialog
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadGreen
import net.mullvad.mullvadvpn.compose.theme.MullvadGreen40
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite80
import net.mullvad.mullvadvpn.util.capitalizeFirstCharOfEachWord
import net.mullvad.mullvadvpn.util.formatDate
import net.mullvad.mullvadvpn.viewmodel.DeviceListViewModel

@Composable
fun DeviceListScreen(
    viewModel: DeviceListViewModel,
    onBackClick: () -> Unit,
    onContinueWithLogin: () -> Unit
) {
    val state = viewModel.uiState.collectAsState().value

    if (state.stagedDevice != null) {
        ShowDeviceRemovalDialog(viewModel = viewModel, device = state.stagedDevice)
    }

    ConstraintLayout(
        modifier =
            Modifier.fillMaxHeight().fillMaxWidth().background(MaterialTheme.colorScheme.secondary)
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
                                top.linkTo(parent.top, margin = 30.dp)
                                start.linkTo(parent.start)
                                end.linkTo(parent.end)
                            }
                            .width(64.dp)
                            .height(64.dp)
                )

                Column(
                    modifier =
                        Modifier.constrainAs(message) {
                            top.linkTo(icon.bottom, margin = 16.dp)
                            start.linkTo(parent.start, margin = 22.dp)
                            end.linkTo(parent.end, margin = 22.dp)
                            width = Dimension.fillToConstraints
                        },
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
                        style = MaterialTheme.typography.headlineSmall
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
                        modifier =
                            Modifier.wrapContentHeight().animateContentSize().padding(top = 8.dp)
                    )
                }

                Box(
                    modifier =
                        Modifier.constrainAs(list) {
                            top.linkTo(message.bottom, margin = 20.dp)
                            height = Dimension.wrapContent
                            width = Dimension.matchParent
                        }
                ) {
                    Column {
                        state.deviceUiItems.forEach { deviceUiState ->
                            ListItem(
                                text = deviceUiState.device.name.capitalizeFirstCharOfEachWord(),
                                subText =
                                    deviceUiState.device.creationDate?.let { creationDate ->
                                        stringResource(
                                            id = R.string.created_x,
                                            creationDate.formatDate()
                                        )
                                    },
                                height = Dimens.listItemHeightExtra,
                                isLoading = deviceUiState.isLoading,
                                iconResourceId = R.drawable.icon_close
                            ) {
                                viewModel.stageDeviceForRemoval(deviceUiState.device.id)
                            }
                        }
                    }
                }
            }
        }

        Column(
            modifier =
                Modifier.constrainAs(buttons) {
                    bottom.linkTo(parent.bottom, margin = 22.dp)
                    start.linkTo(parent.start, margin = 22.dp)
                    end.linkTo(parent.end, margin = 22.dp)
                    width = Dimension.fillToConstraints
                }
        ) {
            ActionButton(
                text = stringResource(id = R.string.continue_login),
                onClick = onContinueWithLogin,
                isEnabled = state.hasTooManyDevices.not() && state.isLoading.not(),
                colors =
                    ButtonDefaults.buttonColors(
                        backgroundColor = MullvadGreen,
                        disabledBackgroundColor = MullvadGreen40,
                        disabledContentColor = MullvadWhite80,
                        contentColor = MullvadWhite
                    )
            )

            ActionButton(
                text = stringResource(id = R.string.back),
                onClick = onBackClick,
                colors =
                    ButtonDefaults.buttonColors(
                        backgroundColor = MullvadBlue,
                        contentColor = MullvadWhite
                    ),
                modifier = Modifier.padding(top = 16.dp)
            )
        }
    }
}
