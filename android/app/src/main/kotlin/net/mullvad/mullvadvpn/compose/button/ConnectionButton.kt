package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LocalMinimumInteractiveComponentEnforcement
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import androidx.constraintlayout.compose.Dimension
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisconnectButton
import net.mullvad.mullvadvpn.lib.theme.color.onVariant
import net.mullvad.mullvadvpn.lib.theme.color.variant
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
            MaterialTheme.colorScheme.variant
        } else {
            MaterialTheme.colorScheme.error.copy(alpha = AlphaDisconnectButton)
        }

    val contentColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.onVariant
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
private fun PreviewConnectionButton() {
    AppTheme {
        ConnectionButton(
            text = "Disconnect",
            mainClick = {},
            containerColor = MaterialTheme.colorScheme.error.copy(alpha = AlphaDisconnectButton),
            contentColor = MaterialTheme.colorScheme.onError,
            reconnectClick = {},
            isReconnectButtonEnabled = true
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ConnectionButton(
    text: String,
    mainClick: () -> Unit,
    reconnectClick: () -> Unit,
    isReconnectButtonEnabled: Boolean,
    containerColor: Color,
    contentColor: Color,
    modifier: Modifier = Modifier,
    reconnectButtonTestTag: String = ""
) {
    ConstraintLayout(
        modifier = modifier.padding(vertical = Dimens.connectButtonExtraPadding).fillMaxWidth()
    ) {
        // initial height set at 0.dp
        var componentHeight by remember { mutableStateOf(0.dp) }

        // get local density from composable
        val density = LocalDensity.current

        val (connectionButton, reconnectButton) = createRefs()
        CompositionLocalProvider(
            LocalMinimumInteractiveComponentEnforcement provides false,
        ) {
            val dividerSize = Dimens.listItemDivider

            Button(
                onClick = mainClick,
                shape =
                    if (isReconnectButtonEnabled) {
                        MaterialTheme.shapes.small.copy(
                            topEnd = CornerSize(percent = 0),
                            bottomEnd = CornerSize(percent = 0)
                        )
                    } else {
                        MaterialTheme.shapes.small
                    },
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = containerColor,
                        contentColor = contentColor
                    ),
                modifier =
                    Modifier.constrainAs(connectionButton) {
                            start.linkTo(parent.start)
                            if (isReconnectButtonEnabled) {
                                end.linkTo(reconnectButton.start)
                            } else {
                                end.linkTo(parent.end)
                            }
                            width = Dimension.fillToConstraints
                            height = Dimension.wrapContent
                        }
                        .onGloballyPositioned {
                            componentHeight = with(density) { it.size.height.toDp() }
                        }
            ) {
                // Offset to compensate for the reconnect button.
                Text(
                    text = text,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.Bold,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier =
                        if (isReconnectButtonEnabled) {
                            Modifier.padding(start = componentHeight + Dimens.listItemDivider)
                        } else {
                            Modifier
                        }
                )
            }

            if (isReconnectButtonEnabled) {
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
                        Modifier.testTag(reconnectButtonTestTag)
                            .constrainAs(reconnectButton) {
                                start.linkTo(connectionButton.end, margin = dividerSize)
                                top.linkTo(connectionButton.top)
                                bottom.linkTo(connectionButton.bottom)
                                end.linkTo(parent.end)
                                height = Dimension.fillToConstraints
                            }
                            .aspectRatio(1f, true)
                ) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_reload),
                        contentDescription = null
                    )
                }
            }
        }
    }
}
