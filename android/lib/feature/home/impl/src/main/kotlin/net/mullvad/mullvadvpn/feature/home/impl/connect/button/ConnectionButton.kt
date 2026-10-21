package net.mullvad.mullvadvpn.feature.home.impl.connect.button

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import net.mullvad.mullvadvpn.feature.home.impl.connect.TunnelStatePreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.TunnelState
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive

@Composable
@Preview
private fun PreviewConnectionButton(
    @PreviewParameter(TunnelStatePreviewParameterProvider::class) tunnelState: TunnelState
) {
    AppTheme {
        Column(verticalArrangement = Arrangement.spacedBy(Dimens.mediumSpacer)) {
            ConnectionButton(
                state = tunnelState,
                disconnectClick = {},
                cancelClick = {},
                connectClick = {},
            )
            ConnectionButton(
                state = tunnelState,
                disconnectClick = {},
                cancelClick = {},
                connectClick = {},
                isPauseButtonEnabled = true,
            )
        }
    }
}

@Composable
fun ConnectionButton(
    modifier: Modifier = Modifier,
    state: TunnelState,
    disconnectClick: () -> Unit,
    cancelClick: () -> Unit,
    connectClick: () -> Unit,
    pauseClick: () -> Unit = {},
    isPauseButtonEnabled: Boolean = false,
    pauseButtonTestTag: String = "",
) {

    val containerColor =
        if (state is TunnelState.Disconnected) {
            MaterialTheme.colorScheme.positive
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
                            R.string.disconnect
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

    if (isPauseButtonEnabled) {
        ConnectionButtonWithPause(
            modifier = modifier,
            text = buttonText,
            containerColor = containerColor,
            contentColor = contentColor,
            onClick = onClick,
            pauseClick = pauseClick,
            pauseButtonTestTag = pauseButtonTestTag,
        )
    } else {
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
}

// Follows the layout of SwitchLocationButton, so that both buttons line up when connected.
@Composable
private fun ConnectionButtonWithPause(
    text: String,
    containerColor: Color,
    contentColor: Color,
    onClick: () -> Unit,
    pauseClick: () -> Unit,
    pauseButtonTestTag: String,
    modifier: Modifier = Modifier,
) {
    CompositionLocalProvider(
        LocalMinimumInteractiveComponentSize provides
            Dimens.reconnectButtonMinInteractiveComponentSize
    ) {
        Row(
            modifier =
                modifier
                    .padding(vertical = Dimens.connectButtonExtraPadding)
                    .height(IntrinsicSize.Min),
            horizontalArrangement = Arrangement.spacedBy(Dimens.reconnectButtonDivider),
        ) {
            Button(
                onClick = onClick,
                shape = RoundedCornerShape(topStartPercent = 50, bottomStartPercent = 50),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = containerColor,
                        contentColor = contentColor,
                    ),
                modifier = Modifier.weight(1f),
            ) {
                Text(
                    text = text,
                    textAlign = TextAlign.Center,
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    // Offset to keep the text centered over the full width, including the pause
                    // button.
                    modifier =
                        Modifier.padding(
                            start =
                                Dimens.switchLocationRetryMinWidth + Dimens.reconnectButtonDivider
                        ),
                )
            }

            FilledIconButton(
                onClick = pauseClick,
                shape = RoundedCornerShape(topEndPercent = 50, bottomEndPercent = 50),
                colors =
                    IconButtonDefaults.filledIconButtonColors(
                        containerColor = containerColor,
                        contentColor = contentColor,
                    ),
                modifier =
                    Modifier.testTag(pauseButtonTestTag)
                        .fillMaxHeight()
                        .defaultMinSize(minWidth = Dimens.switchLocationRetryMinWidth),
            ) {
                Icon(
                    imageVector = Icons.Rounded.Pause,
                    contentDescription = stringResource(id = R.string.pause_vpn),
                )
            }
        }
    }
}
