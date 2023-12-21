package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.typeface.connectionStatus
import net.mullvad.mullvadvpn.model.TunnelState
import net.mullvad.talpid.tunnel.ActionAfterDisconnect
import net.mullvad.talpid.tunnel.ErrorState
import net.mullvad.talpid.tunnel.ErrorStateCause

@Preview
@Composable
private fun PreviewConnectionStatusText() {
    AppTheme {
        SpacedColumn {
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
        color = MaterialTheme.colorScheme.onPrimary,
        style = MaterialTheme.typography.connectionStatus,
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
        color = MaterialTheme.colorScheme.surface,
        style = MaterialTheme.typography.connectionStatus,
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
            if (isBlocking) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.error,
        style = MaterialTheme.typography.connectionStatus,
        modifier = modifier
    )
}
