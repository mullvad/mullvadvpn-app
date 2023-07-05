package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.scrollable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ConnectionButton
import net.mullvad.mullvadvpn.compose.button.SwitchLocationButton
import net.mullvad.mullvadvpn.compose.component.ConnectionStatusText
import net.mullvad.mullvadvpn.compose.component.LocationInfo
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
    val scrollState = rememberScrollState()
    Column(
        verticalArrangement = Arrangement.Bottom,
        horizontalAlignment = Alignment.Start,
        modifier =
            Modifier.background(color = MaterialTheme.colorScheme.primary)
                .fillMaxHeight()
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
                .scrollable(scrollState, Orientation.Vertical)
    ) {
        if (
            uiState.tunnelRealState is TunnelState.Connecting ||
                (uiState.tunnelRealState is TunnelState.Disconnecting &&
                    uiState.tunnelRealState.actionAfterDisconnect ==
                        ActionAfterDisconnect.Reconnect)
        ) {
            CircularProgressIndicator(
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.size(
                            width = Dimens.progressIndicatorSize,
                            height = Dimens.progressIndicatorSize
                        )
                        .align(Alignment.CenterHorizontally)
            )
        }
        Spacer(modifier = Modifier.height(Dimens.smallPadding))
        ConnectionStatusText(state = uiState.tunnelRealState)
        Text(
            text = uiState.location?.country ?: "",
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary
        )
        Text(
            text = uiState.location?.city ?: "",
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary
        )
        LocationInfo(
            onToggleTunnelInfo = onToggleTunnelInfo,
            isVisible = uiState.tunnelRealState != TunnelState.Disconnected,
            isExpanded = uiState.isTunnelInfoExpanded,
            location = uiState.location,
            inAddress = uiState.inAddress,
            outAddress = uiState.outAddress,
            modifier = Modifier.fillMaxWidth()
        )
        Spacer(modifier = Modifier.height(Dimens.buttonSeparation))
        SwitchLocationButton(
            modifier = Modifier.fillMaxWidth().height(Dimens.selectLocationButtonHeight),
            onClick = onSwitchLocationClick,
            showChevron = uiState.showLocation,
            text =
                if (uiState.showLocation) {
                    uiState.relayLocation?.locationName ?: ""
                } else {
                    stringResource(id = R.string.switch_location)
                }
        )
        Spacer(modifier = Modifier.height(Dimens.buttonSeparation))
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
