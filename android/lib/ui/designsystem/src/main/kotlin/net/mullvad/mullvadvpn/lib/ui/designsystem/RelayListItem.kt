package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.Stable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.RectangleShape
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewFontScale
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInactive

@Composable
fun RelayListItem(
    modifier: Modifier = Modifier,
    selected: Boolean = false,
    enabled: Boolean = true,
    onClick: (() -> Unit) = {},
    onLongClick: (() -> Unit)? = {},
    content: @Composable () -> Unit,
    trailingContent: @Composable (() -> Unit)? = null,
    colors: RelayListItemColors = RelayListItemDefaults.colors(),
    shape: Shape = RectangleShape,
) {
    Surface(
        modifier =
            modifier
                .defaultMinSize(minHeight = RelayListTokens.listItemMinHeight)
                .height(IntrinsicSize.Min),
        shape = shape,
    ) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(RelayListTokens.listItemSpacer),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Row(
                Modifier.weight(1f, fill = true)
                    .background(colors.containerColor)
                    .fillMaxHeight()
                    .combinedClickable(
                        enabled = true,
                        onClick = onClick,
                        onLongClick = onLongClick,
                    ),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                ProvideContentColorTextStyle(
                    colors.headlineColor(enabled, selected),
                    MaterialTheme.typography.titleMedium,
                ) {
                    content()
                }
            }

            if (trailingContent != null) {
                Box(
                    Modifier.background(color = colors.containerColor)
                        .width(RelayListTokens.listItemButtonWidth)
                        .fillMaxHeight()
                ) {
                    ProvideContentColorTextStyle(
                        colors.trailingIconColor,
                        MaterialTheme.typography.titleMedium,
                    ) {
                        trailingContent()
                    }
                }
            }
        }
    }
}

// Based of ListItem
@Immutable
class RelayListItemColors(
    val containerColor: Color,
    val headlineColor: Color,
    val trailingIconColor: Color,
    val selectedHeadlineColor: Color,
    val disabledHeadlineColor: Color,
) {
    internal fun containerColor(): Color = containerColor

    @Stable
    internal fun headlineColor(enabled: Boolean, selected: Boolean): Color =
        when {
            !enabled -> disabledHeadlineColor
            selected -> selectedHeadlineColor
            else -> headlineColor
        }
}

@Composable
internal fun ProvideContentColorTextStyle(
    contentColor: Color,
    textStyle: TextStyle,
    content: @Composable () -> Unit,
) {
    val mergedStyle = LocalTextStyle.current.merge(textStyle)
    CompositionLocalProvider(
        LocalContentColor provides contentColor,
        LocalTextStyle provides mergedStyle,
        content = content,
    )
}

object RelayListItemDefaults {
    @Composable
    fun colors(
        containerColor: Color = MaterialTheme.colorScheme.surface,
        headlineColor: Color = MaterialTheme.colorScheme.onSurface,
        trailingIconColor: Color = MaterialTheme.colorScheme.onSurface,
        selectedHeadlineColor: Color = MaterialTheme.colorScheme.tertiary,
        disabledHeadlineColor: Color =
            headlineColor.copy(alpha = RelayListTokens.RelayListItemDisabledLabelTextOpacity),
    ): RelayListItemColors =
        RelayListItemColors(
            containerColor = containerColor,
            headlineColor = headlineColor,
            trailingIconColor = trailingIconColor,
            selectedHeadlineColor = selectedHeadlineColor,
            disabledHeadlineColor = disabledHeadlineColor,
        )
}

object RelayListTokens {
    const val RelayListItemDisabledLabelTextOpacity = AlphaInactive

    val listItemMinHeight = 56.dp
    val listItemSpacer = 2.dp
    val listItemButtonWidth = 56.dp
}

@Preview
@PreviewFontScale
@Composable
private fun PreviewSimpleRelayListItem() {
    AppTheme {
        RelayListItem(
            modifier = Modifier.fillMaxWidth(),
            content = { Text("Hello world", modifier = Modifier.padding(16.dp).fillMaxSize()) },
        )
    }
}

@Preview
@PreviewFontScale
@Composable
private fun PreviewLeadingRelayListItem() {
    AppTheme {
        RelayListItem(
            modifier = Modifier.fillMaxWidth(),
            content = {
                Text(
                    "Hello world fsadhkuhfiuskahf iuhsadhuf sa",
                    modifier =
                        Modifier.padding(16.dp)
                            .fillMaxSize()
                            .wrapContentHeight(align = Alignment.CenterVertically),
                    style = MaterialTheme.typography.titleMedium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            },
            trailingContent = {
                Box(
                    modifier = Modifier.fillMaxSize().clickable(onClick = { /* Handle click */ }),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        modifier = Modifier.padding(16.dp),
                        imageVector = Icons.Default.KeyboardArrowDown,
                        contentDescription = null,
                    )
                }
            },
        )
    }
}

@Preview
@PreviewFontScale
@Composable
private fun PreviewTrailingRelayListItem() {
    AppTheme {
        RelayListItem(
            modifier = Modifier.fillMaxWidth(),
            selected = true,
            content = {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(imageVector = Icons.Default.Check, contentDescription = null)
                    Spacer(Modifier.width(8.dp))
                    Text("Sample Relay Item", maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            },
            trailingContent = {
                Box(
                    modifier = Modifier.fillMaxSize().clickable(onClick = {}),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        modifier = Modifier.padding(16.dp),
                        imageVector = Icons.Default.Add,
                        contentDescription = null,
                    )
                }
            },
        )
    }
}
