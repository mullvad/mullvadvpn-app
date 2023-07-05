package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ConnectionButton
import net.mullvad.mullvadvpn.compose.component.ConnectionStatusText
import net.mullvad.mullvadvpn.compose.component.LocationInfo
import net.mullvad.mullvadvpn.compose.component.SwitchLocationButton
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

@Preview
@Composable
fun PreviewConnectScreen() {
    val state = ConnectUiState.INITIAL
    AppTheme { ConnectScreen(state) }
}

@Composable
fun ConnectScreen(
    uiState: ConnectUiState,
    onDisconnectClick: () -> Unit = {},
    onReconnectClick: () -> Unit = {},
    onConnectClick: () -> Unit = {},
    onCancelClick: () -> Unit = {},
    onSwitchLocationClick: () -> Unit = {},
    onToggleTunnelInfo: () -> Unit = {}
) {
    LazyColumn(
        verticalArrangement = Arrangement.Bottom,
        horizontalAlignment = Alignment.Start,
        modifier =
            Modifier.background(color = MaterialTheme.colorScheme.primary)
                .fillMaxHeight()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
    ) {
        if (
            uiState.tunnelRealState is TunnelState.Connecting ||
                (uiState.tunnelRealState is TunnelState.Disconnecting &&
                    uiState.tunnelRealState.actionAfterDisconnect ==
                        ActionAfterDisconnect.Reconnect)
        ) {
            item {
                Box(modifier = Modifier.fillMaxWidth().padding(bottom = Dimens.smallPadding)) {
                    CircularProgressIndicator(
                        color = MaterialTheme.colorScheme.onPrimary,
                        modifier =
                            Modifier.size(
                                    width = Dimens.progressIndicatorSize,
                                    height = Dimens.progressIndicatorSize
                                )
                                .align(Alignment.Center)
                    )
                }
            }
        }
        item { ConnectionStatusText(state = uiState.tunnelRealState) }
        item {
            Text(
                text = uiState.location?.country ?: "",
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary
            )
        }
        item {
            Text(
                text = uiState.location?.city ?: "",
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary
            )
        }
        item {
            LocationInfo(
                onToggleTunnelInfo = onToggleTunnelInfo,
                visible = uiState.tunnelRealState != TunnelState.Disconnected,
                expanded = uiState.isTunnelInfoExpanded,
                location = uiState.location,
                tunnelEndpoint =
                    when (uiState.tunnelRealState) {
                        is TunnelState.Connected -> uiState.tunnelRealState.endpoint
                        is TunnelState.Connecting -> uiState.tunnelRealState.endpoint
                        else -> null
                    },
                modifier = Modifier.fillMaxWidth()
            )
        }
        item { Spacer(modifier = Modifier.height(Dimens.buttonSeparation)) }
        item {
            val showLocation =
                when (uiState.tunnelUiState) {
                    is TunnelState.Disconnected -> true
                    is TunnelState.Disconnecting -> {
                        when (uiState.tunnelUiState.actionAfterDisconnect) {
                            ActionAfterDisconnect.Nothing -> true
                            ActionAfterDisconnect.Block -> true
                            ActionAfterDisconnect.Reconnect -> false
                        }
                    }
                    is TunnelState.Connecting -> false
                    is TunnelState.Connected -> false
                    is TunnelState.Error -> true
                }
            SwitchLocationButton(
                modifier = Modifier.fillMaxWidth().height(Dimens.selectLocationButtonHeight),
                onClick = onSwitchLocationClick,
                showChevron = showLocation,
                text =
                    if (showLocation) {
                        uiState.relayLocation?.locationName ?: ""
                    } else {
                        stringResource(id = R.string.switch_location)
                    }
            )
        }
        item { Spacer(modifier = Modifier.height(Dimens.buttonSeparation)) }
        item {
            ConnectionButton(
                state = uiState.tunnelUiState,
                modifier = Modifier.fillMaxWidth().height(Dimens.connectButtonHeight),
                disconnectClick = onDisconnectClick,
                reconnectClick = onReconnectClick,
                cancelClick = onCancelClick,
                connectClick = onConnectClick,
            )
        }
    }
}
