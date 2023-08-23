package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
fun PreviewConnectionStatusText() {
    AppTheme {
        SpacedColumn {
            ConnectionStatusText(TunnelState.Disconnected)
            ConnectionStatusText(TunnelState.Connecting(null, null))
            ConnectionStatusText(
                state = TunnelState.Error(ErrorState(ErrorStateCause.Ipv6Unavailable, true))
            )
        }
    }
}

@Composable
fun ConnectionStatusText(state: TunnelState) {
    when (state) {
        is TunnelState.Disconnecting -> {
            when (state.actionAfterDisconnect) {
                ActionAfterDisconnect.Nothing -> DisconnectedText()
                ActionAfterDisconnect.Block -> ConnectedText(false)
                ActionAfterDisconnect.Reconnect -> ConnectingText(false)
            }
        }
        is TunnelState.Disconnected -> DisconnectedText()
        is TunnelState.Connecting -> ConnectingText(state.endpoint?.quantumResistant == true)
        is TunnelState.Connected -> ConnectedText(state.endpoint.quantumResistant)
        is TunnelState.Error -> ErrorText(state.errorState.isBlocking)
    }
}

@Composable
private fun DisconnectedText() {
    Text(
        text = textResource(id = R.string.unsecured_connection),
        color = MaterialTheme.colorScheme.error,
        style = MaterialTheme.typography.connectionStatus
    )
}

@Composable
private fun ConnectingText(isQuantumResistant: Boolean) {
    Text(
        text =
            textResource(
                id =
                    if (isQuantumResistant) R.string.quantum_creating_secure_connection
                    else R.string.creating_secure_connection
            ),
        color = MaterialTheme.colorScheme.onPrimary,
        style = MaterialTheme.typography.connectionStatus
    )
}

@Composable
private fun ConnectedText(isQuantumResistant: Boolean) {
    Text(
        text =
            textResource(
                id =
                    if (isQuantumResistant) R.string.quantum_secure_connection
                    else R.string.secure_connection
            ),
        color = MaterialTheme.colorScheme.surface,
        style = MaterialTheme.typography.connectionStatus
    )
}

@Composable
private fun ErrorText(isBlocking: Boolean) {
    Text(
        text =
            textResource(
                id = if (isBlocking) R.string.blocked_connection else R.string.error_state
            ),
        color =
            if (isBlocking) MaterialTheme.colorScheme.surface else MaterialTheme.colorScheme.error,
        style = MaterialTheme.typography.connectionStatus
    )
}
