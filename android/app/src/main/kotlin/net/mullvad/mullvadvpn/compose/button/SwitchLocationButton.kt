package net.mullvad.mullvadvpn.compose.button

import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.CornerSize
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
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
import net.mullvad.mullvadvpn.compose.component.SpacedColumn
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewConnectionButton() {
    AppTheme {
        SpacedColumn {
            SwitchLocationButton(
                text = "Switch Location",
                onSwitchLocation = {},
                reconnectClick = {},
                isReconnectButtonEnabled = true,
            )
            SwitchLocationButton(
                text = "Switch Location",
                onSwitchLocation = {},
                reconnectClick = {},
                isReconnectButtonEnabled = false,
            )
        }
    }
}

@Composable
@Suppress("LongMethod")
fun SwitchLocationButton(
    text: String,
    onSwitchLocation: () -> Unit,
    reconnectClick: () -> Unit,
    isReconnectButtonEnabled: Boolean,
    modifier: Modifier = Modifier,
    reconnectButtonTestTag: String = "",
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
            LocalMinimumInteractiveComponentSize provides
                Dimens.reconnectButtonMinInteractiveComponentSize
        ) {
            val dividerSize = Dimens.listItemDivider

            Button(
                onClick = onSwitchLocation,
                shape =
                    if (isReconnectButtonEnabled) {
                        MaterialTheme.shapes.small.copy(
                            topEnd = CornerSize(percent = 0),
                            bottomEnd = CornerSize(percent = 0),
                        )
                    } else {
                        MaterialTheme.shapes.small
                    },
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = MaterialTheme.colorScheme.primary,
                        contentColor = MaterialTheme.colorScheme.onPrimary,
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
                        },
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
                            Modifier.padding(
                                start =
                                    componentHeight + Dimens.listItemDivider + Dimens.smallPadding
                            )
                        } else {
                            Modifier
                        },
                )
            }

            if (isReconnectButtonEnabled) {
                FilledIconButton(
                    shape =
                        MaterialTheme.shapes.small.copy(
                            topStart = CornerSize(percent = 0),
                            bottomStart = CornerSize(percent = 0),
                        ),
                    colors =
                        IconButtonDefaults.filledIconButtonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary,
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
                            .defaultMinSize(minWidth = Dimens.switchLocationRetryMinWidth),
                ) {
                    Icon(
                        painter = painterResource(R.drawable.icon_reconnect),
                        contentDescription = stringResource(id = R.string.reconnect),
                    )
                }
            }
        }
    }
}
