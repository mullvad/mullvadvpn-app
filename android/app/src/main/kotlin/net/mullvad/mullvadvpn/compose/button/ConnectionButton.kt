package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AlphaDisconnectButton
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.model.TunnelState

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
    val containerColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.surface
        } else {
            MaterialTheme.colorScheme.error.copy(alpha = AlphaDisconnectButton)
        }

    val contentColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.onSurface
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
                            R.string.disconnect
                        } else {
                            R.string.dismiss
                        }
                    }
                }
        )

    val onMainClick =
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

    ConnectionButton(
        modifier = modifier,
        text = buttonText,
        containerColor = containerColor,
        contentColor = contentColor,
        mainClick = onMainClick,
        reconnectClick = reconnectClick,
        reconnectButtonTestTag = reconnectButtonTestTag,
        isReconnectButtonEnabled = (state is TunnelState.Disconnected).not()
    )
}

@Preview
@Composable
fun ConnectionButton() {
    AppTheme {
        ConnectionButton(
            text = "Disconnect",
            mainClick = {},
            containerColor = MaterialTheme.colorScheme.error.copy(alpha = AlphaDisconnectButton),
            contentColor = MaterialTheme.colorScheme.onError,
            reconnectClick = {},
            isReconnectButtonEnabled = false
        )
    }
}

@Composable
private fun ConnectionButton(
    text: String,
    mainClick: () -> Unit,
    reconnectClick: () -> Unit,
    isReconnectButtonEnabled: Boolean,
    containerColor: Color,
    contentColor: Color,
    modifier: Modifier = Modifier,
    height: Dp = Dimens.connectButtonHeight,
    reconnectButtonTestTag: String = ""
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
                    containerColor = containerColor,
                    contentColor = contentColor
                ),
            modifier = Modifier.weight(1f).height(height)
        ) {
            // Offset to compensate for the reconnect button.
            val paddingOffset =
                if (isReconnectButtonEnabled) {
                    height + Dimens.listItemDivider
                } else {
                    0.dp
                }
            Text(
                text = text,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(start = paddingOffset)
            )
        }

        if (isReconnectButtonEnabled) {
            Spacer(modifier = Modifier.width(Dimens.listItemDivider))

            FilledIconButton(
                shape =
                    MaterialTheme.shapes.small.copy(
                        topStart = CornerSize(percent = 0),
                        bottomStart = CornerSize(percent = 0)
                    ),
                colors =
                    IconButtonDefaults.filledIconButtonColors(
                        containerColor = containerColor,
                        contentColor = contentColor
                    ),
                onClick = reconnectClick,
                modifier =
                    Modifier.height(height).aspectRatio(1f, true).testTag(reconnectButtonTestTag)
            ) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_reload),
                    contentDescription = null
                )
            }
        }
    }
}
