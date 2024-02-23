package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.ErrorState
import net.mullvad.mullvadvpn.lib.model.ErrorStateCause
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.typeface.connectionStatus

@Preview
@Composable
private fun PreviewConnectionStatusText() {
    AppTheme {
        SpacedColumn(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            ConnectionStatusText(TunnelState.Disconnected())
            ConnectionStatusText(TunnelState.Connecting(null, null))
            ConnectionStatusText(
                state = TunnelState.Error(ErrorState(ErrorStateCause.Ipv6Unavailable, true))
            )
        }
    }
}

@Composable
fun ConnectionStatusText(state: TunnelState, modifier: Modifier = Modifier) {
    when (state) {
        is TunnelState.Disconnecting -> {
            when (state.actionAfterDisconnect) {
                ActionAfterDisconnect.Nothing -> DisconnectedText(modifier = modifier)
                ActionAfterDisconnect.Block ->
                    ConnectedText(isQuantumResistant = false, modifier = modifier)
                ActionAfterDisconnect.Reconnect ->
                    ConnectingText(isQuantumResistant = false, modifier = modifier)
            }
        }
        is TunnelState.Disconnected -> DisconnectedText(modifier = modifier)
        is TunnelState.Connecting ->
            ConnectingText(
                isQuantumResistant = state.endpoint?.quantumResistant == true,
                modifier = modifier
            )
        is TunnelState.Connected ->
            ConnectedText(isQuantumResistant = state.endpoint.quantumResistant, modifier = modifier)
        is TunnelState.Error ->
            ErrorText(isBlocking = state.errorState.isBlocking, modifier = modifier)
    }
}

@Composable
private fun DisconnectedText(modifier: Modifier) {
    Text(
        text = textResource(id = R.string.unsecured_connection),
        color = MaterialTheme.colorScheme.error,
        style = MaterialTheme.typography.connectionStatus,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
    )
}

@Composable
private fun ConnectingText(isQuantumResistant: Boolean, modifier: Modifier) {
    Text(
        text =
            textResource(
                id =
                    if (isQuantumResistant) R.string.quantum_creating_secure_connection
                    else R.string.creating_secure_connection
            ),
        color = MaterialTheme.colorScheme.onSurface,
        style = MaterialTheme.typography.connectionStatus,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
    )
}

@Composable
private fun ConnectedText(isQuantumResistant: Boolean, modifier: Modifier) {
    Text(
        text =
            textResource(
                id =
                    if (isQuantumResistant) R.string.quantum_secure_connection
                    else R.string.secure_connection
            ),
        color = MaterialTheme.colorScheme.tertiary,
        style = MaterialTheme.typography.connectionStatus,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
    )
}

@Composable
private fun ErrorText(isBlocking: Boolean, modifier: Modifier) {
    Text(
        text =
            textResource(
                id = if (isBlocking) R.string.blocked_connection else R.string.error_state
            ),
        color =
            if (isBlocking) MaterialTheme.colorScheme.onSurface
            else MaterialTheme.colorScheme.error,
        style = MaterialTheme.typography.connectionStatus,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        modifier = modifier
    )
}
