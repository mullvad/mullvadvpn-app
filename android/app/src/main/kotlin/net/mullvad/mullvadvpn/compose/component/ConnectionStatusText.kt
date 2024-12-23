package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.ActionAfterDisconnect
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.typeface.connectionStatus

@Preview
@Composable
private fun PreviewConnectionStatusText(
    @PreviewParameter(TunnelStatePreviewParameterProvider::class) tunnelState: TunnelState
) {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            ConnectionStatusText(state = tunnelState)
        }
    }
}

@Composable
fun ConnectionStatusText(state: TunnelState) {
    Text(
        text = state.text(),
        color = state.textColor(),
        style = MaterialTheme.typography.connectionStatus,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun TunnelState.text() =
    when (this) {
        is TunnelState.Connected -> textResource(id = R.string.connected)
        is TunnelState.Connecting -> textResource(id = R.string.connecting)
        is TunnelState.Disconnected -> textResource(id = R.string.disconnected)
        is TunnelState.Disconnecting ->
            when (actionAfterDisconnect) {
                ActionAfterDisconnect.Nothing -> textResource(id = R.string.disconnecting)
                ActionAfterDisconnect.Block -> textResource(id = R.string.blocking)
                ActionAfterDisconnect.Reconnect -> textResource(id = R.string.connecting)
            }
        is TunnelState.Error ->
            textResource(
                id =
                    if (errorState.isBlocking) R.string.blocked_connection else R.string.error_state
            )
    }.uppercase()

@Composable
private fun TunnelState.textColor() =
    when (this) {
        is TunnelState.Connected -> MaterialTheme.colorScheme.tertiary
        is TunnelState.Connecting -> MaterialTheme.colorScheme.onSurface
        is TunnelState.Disconnected -> MaterialTheme.colorScheme.error
        is TunnelState.Disconnecting ->
            when (actionAfterDisconnect) {
                ActionAfterDisconnect.Nothing -> MaterialTheme.colorScheme.error
                ActionAfterDisconnect.Block -> MaterialTheme.colorScheme.tertiary
                ActionAfterDisconnect.Reconnect -> MaterialTheme.colorScheme.onSurface
            }
        is TunnelState.Error ->
            if (errorState.isBlocking) MaterialTheme.colorScheme.onSurface
            else MaterialTheme.colorScheme.error
    }
