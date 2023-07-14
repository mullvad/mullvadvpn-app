package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect

@Composable
fun ConnectionButton(
    modifier: Modifier = Modifier,
    reconnectButtonTestTag: String = "",
    state: TunnelState,
    disconnectClick: () -> Unit,
    reconnectClick: () -> Unit,
    cancelClick: () -> Unit,
    connectClick: () -> Unit
) {
    when (state) {
        is TunnelState.Disconnected -> ConnectButton(modifier = modifier, onClick = connectClick)
        is TunnelState.Disconnecting -> {
            when (state.actionAfterDisconnect) {
                ActionAfterDisconnect.Nothing ->
                    ConnectButton(modifier = modifier, onClick = connectClick)
                ActionAfterDisconnect.Block ->
                    DisconnectButton(
                        modifier = modifier,
                        text = stringResource(id = R.string.disconnect),
                        mainClick = connectClick,
                        reconnectClick = reconnectClick,
                        reconnectButtonTestTag = reconnectButtonTestTag
                    )
                ActionAfterDisconnect.Reconnect ->
                    DisconnectButton(
                        modifier = modifier,
                        text = stringResource(id = R.string.disconnect),
                        mainClick = connectClick,
                        reconnectClick = reconnectClick,
                        reconnectButtonTestTag = reconnectButtonTestTag
                    )
            }
        }
        is TunnelState.Connecting ->
            DisconnectButton(
                modifier = modifier,
                text = stringResource(id = R.string.cancel),
                mainClick = cancelClick,
                reconnectClick = reconnectClick,
                reconnectButtonTestTag = reconnectButtonTestTag
            )
        is TunnelState.Connected ->
            DisconnectButton(
                modifier = modifier,
                text = stringResource(id = R.string.disconnect),
                mainClick = disconnectClick,
                reconnectClick = reconnectClick,
                reconnectButtonTestTag = reconnectButtonTestTag
            )
        is TunnelState.Error -> {
            if (state.errorState.isBlocking) {
                DisconnectButton(
                    modifier = modifier,
                    text = stringResource(id = R.string.disconnect),
                    mainClick = disconnectClick,
                    reconnectClick = reconnectClick,
                    reconnectButtonTestTag = reconnectButtonTestTag
                )
            } else {
                DisconnectButton(
                    modifier = modifier,
                    text = stringResource(id = R.string.dismiss),
                    mainClick = cancelClick,
                    reconnectClick = reconnectClick,
                    reconnectButtonTestTag = reconnectButtonTestTag
                )
            }
        }
    }
}

@Preview
@Composable
private fun PreviewConnectButton() {
    AppTheme { ConnectButton(onClick = {}) }
}

@Composable
private fun ConnectButton(modifier: Modifier = Modifier, onClick: () -> Unit) {
    ActionButton(
        text = textResource(id = R.string.connect),
        modifier = modifier,
        onClick = onClick,
        colors =
            ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onSurface
            )
    )
}

@Preview
@Composable
fun PreviewDisconnectButton() {
    AppTheme { DisconnectButton(text = "Disconnect", mainClick = {}, reconnectClick = {}) }
}

@Composable
private fun DisconnectButton(
    text: String,
    modifier: Modifier = Modifier,
    height: Dp = Dimens.connectButtonHeight,
    reconnectButtonTestTag: String = "",
    mainClick: () -> Unit,
    reconnectClick: () -> Unit
) {
    Row(modifier = modifier.height(height)) {
        Button(
            onClick = mainClick,
            shape =
                MaterialTheme.shapes.small.copy(
                    topEnd = CornerSize(percent = 0),
                    bottomEnd = CornerSize(percent = 0)
                ),
            colors =
                ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.error.copy(alpha = AlphaInactive),
                    contentColor = MaterialTheme.colorScheme.onError
                ),
            modifier = Modifier.weight(1f).height(height)
        ) {
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold
            )
        }

        Spacer(modifier = Modifier.width(Dimens.listItemDivider))

        FilledIconButton(
            shape =
                MaterialTheme.shapes.small.copy(
                    topStart = CornerSize(percent = 0),
                    bottomStart = CornerSize(percent = 0)
                ),
            colors =
                IconButtonDefaults.filledIconButtonColors(
                    containerColor = MaterialTheme.colorScheme.error.copy(alpha = AlphaInactive),
                    contentColor = MaterialTheme.colorScheme.onError
                ),
            onClick = reconnectClick,
            modifier = Modifier.height(height).aspectRatio(1f, true).testTag(reconnectButtonTestTag)
        ) {
            Icon(painter = painterResource(id = R.drawable.icon_reload), contentDescription = null)
        }
    }
}
