package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ConnectionButton
import net.mullvad.mullvadvpn.compose.button.SwitchLocationButton
import net.mullvad.mullvadvpn.compose.component.ConnectionStatusText
import net.mullvad.mullvadvpn.compose.component.LocationInfo
import net.mullvad.mullvadvpn.compose.state.ConnectUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.CONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LOCATION_INFO_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.RECONNECT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

private const val CONNECT_BUTTON_THROTTLE_MILLIS = 1000

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
    var lastConnectionActionTimestamp by remember { mutableStateOf(0L) }

    fun handleThrottledAction(action: () -> Unit) {
        val currentTime = System.currentTimeMillis()
        if ((currentTime - lastConnectionActionTimestamp) > CONNECT_BUTTON_THROTTLE_MILLIS) {
            lastConnectionActionTimestamp = currentTime
            action.invoke()
        }
    }

    Column(
        verticalArrangement = Arrangement.Bottom,
        horizontalAlignment = Alignment.Start,
        modifier =
            Modifier.background(color = MaterialTheme.colorScheme.primary)
                .height(IntrinsicSize.Max)
                .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin)
                .verticalScroll(scrollState)
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
                        .testTag(CIRCULAR_PROGRESS_INDICATOR)
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
            isVisible =
                uiState.tunnelRealState != TunnelState.Disconnected &&
                    uiState.location?.hostname != null,
            isExpanded = uiState.isTunnelInfoExpanded,
            location = uiState.location,
            inAddress = uiState.inAddress,
            outAddress = uiState.outAddress,
            modifier = Modifier.fillMaxWidth().testTag(LOCATION_INFO_TEST_TAG)
        )
        Spacer(modifier = Modifier.height(Dimens.buttonSeparation))
        SwitchLocationButton(
            modifier =
                Modifier.fillMaxWidth()
                    .height(Dimens.selectLocationButtonHeight)
                    .testTag(SELECT_LOCATION_BUTTON_TEST_TAG),
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
            modifier =
                Modifier.fillMaxWidth()
                    .height(Dimens.connectButtonHeight)
                    .testTag(CONNECT_BUTTON_TEST_TAG),
            disconnectClick = onDisconnectClick,
            reconnectClick = { handleThrottledAction(onReconnectClick) },
            cancelClick = onCancelClick,
            connectClick = { handleThrottledAction(onConnectClick) },
            reconnectButtonTestTag = RECONNECT_BUTTON_TEST_TAG
        )
    }
}
