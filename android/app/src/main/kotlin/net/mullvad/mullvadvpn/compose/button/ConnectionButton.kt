package net.mullvad.mullvadvpn.compose.button

import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.preview.TunnelStatePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Composable
@Preview
private fun PreviewConnectionButton(
    @PreviewParameter(TunnelStatePreviewParameterProvider::class) tunnelState: TunnelState
) {
    AppTheme {
        ConnectionButton(
            state = tunnelState,
            disconnectClick = {},
            cancelClick = {},
            connectClick = {},
        )
    }
}

@Composable
fun ConnectionButton(
    modifier: Modifier = Modifier,
    state: TunnelState,
    disconnectClick: () -> Unit,
    cancelClick: () -> Unit,
    connectClick: () -> Unit,
) {

    val containerColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.tertiary
        } else {
            MaterialTheme.colorScheme.error
        }

    val contentColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.onTertiary
        } else {
            MaterialTheme.colorScheme.onError
        }

    val buttonText =
        stringResource(
            id =
                when (state) {
                    is TunnelState.Disconnected -> R.string.connect
                    is TunnelState.Disconnecting -> R.string.disconnect
                    is TunnelState.Connecting -> R.string.cancel
                    is TunnelState.Connected -> R.string.disconnect
                    is TunnelState.Error -> {
                        if (state.errorState.isBlocking) {
                            R.string.unblock
                        } else {
                            R.string.dismiss
                        }
                    }
                }
        )

    val onClick =
        when (state) {
            is TunnelState.Disconnected -> connectClick
            is TunnelState.Connecting -> cancelClick
            is TunnelState.Error -> {
                if (state.errorState.isBlocking) {
                    disconnectClick
                } else {
                    cancelClick
                }
            }

            else -> disconnectClick
        }

    PrimaryButton(
        onClick = onClick,
        colors =
            ButtonDefaults.buttonColors(
                containerColor = containerColor,
                contentColor = contentColor,
            ),
        modifier = modifier,
        text = buttonText,
    )
}
