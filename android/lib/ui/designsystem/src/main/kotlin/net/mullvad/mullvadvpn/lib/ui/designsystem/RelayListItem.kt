package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.material3.ListItem
import androidx.compose.material3.ListItemColors
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.Stable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewFontScale
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.theme.AppTheme

private val LIST_ITEM_MIN_HEIGHT = 56.dp
private val LIST_ITEM_SPACER = 2.dp
private val LIST_ITEM_BUTTON_WIDTH = 56.dp

@Composable
fun RelayListItem(
    modifier: Modifier = Modifier,
    selected: Boolean = false,
    enabled: Boolean = true,
    onClick: (() -> Unit) = {},
    onLongClick: (() -> Unit)? = {},
    leadingContent: @Composable (() -> Unit)? = null,
    content: @Composable () -> Unit,
    trailingContent: @Composable (() -> Unit)? = null,
    colors: RelayListItemColors = RelayListItemDefaults.colors(),
) {
    ProvideContentColorTextStyle(colors.headlineColor, MaterialTheme.typography.titleMedium) {
        Row(
            modifier =
                modifier.defaultMinSize(minHeight = LIST_ITEM_MIN_HEIGHT).height(IntrinsicSize.Min),
            horizontalArrangement = Arrangement.spacedBy(LIST_ITEM_SPACER),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (leadingContent != null) {
                Box(
                    Modifier.background(colors.containerColor)
                        .width(LIST_ITEM_BUTTON_WIDTH)
                        .fillMaxHeight(),
                    contentAlignment = Alignment.Center,
                ) {
                    ProvideContentColorTextStyle(
                        colors.leadingIconColor(enabled),
                        MaterialTheme.typography.titleMedium,
                    ) {
                        leadingContent()
                    }
                }
            }

            Row(
                Modifier.weight(1f, fill = true)
                    .background(colors.containerColor)
                    .fillMaxHeight()
                    .combinedClickable(
                        enabled = enabled,
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
                        .width(LIST_ITEM_BUTTON_WIDTH)
                        .fillMaxHeight()
                ) {
                    ProvideContentColorTextStyle(
                        colors.trailingIconColor(enabled),
                        MaterialTheme.typography.titleMedium,
                    ) {
                        trailingContent()
                    }
                }
            }
        }
    }
}

@Immutable
class RelayListItemColors(
    val containerColor: Color,
    val headlineColor: Color,
    val leadingIconColor: Color,
    val trailingIconColor: Color,
    val selectedHeadlineColor: Color,
    val disabledHeadlineColor: Color,
    val disabledLeadingIconColor: Color,
    val disabledTrailingIconColor: Color,
) {
    /** The container color of this [ListItem] based on enabled state */
    internal fun containerColor(): Color = containerColor

    /** The color of this [ListItem]'s headline text based on enabled state */
    @Stable
    internal fun headlineColor(enabled: Boolean, selected: Boolean): Color =
        when {
            !enabled -> disabledHeadlineColor
            selected -> selectedHeadlineColor
            else -> headlineColor
        }

    /** The color of this [ListItem]'s leading content based on enabled state */
    @Stable
    internal fun leadingIconColor(enabled: Boolean): Color =
        if (enabled) leadingIconColor else disabledLeadingIconColor

    /** The color of this [ListItem]'s trailing content based on enabled state */
    @Stable
    internal fun trailingIconColor(enabled: Boolean): Color =
        if (enabled) trailingIconColor else disabledTrailingIconColor
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
    /** The container color of a list item */
    val containerColor: Color
        @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.primaryContainer

    /** The content color of a list item */
    val contentColor: Color
        @Composable @ReadOnlyComposable get() = MaterialTheme.colorScheme.onPrimaryContainer

    /**
     * Creates a [ListItemColors] that represents the default container and content colors used in a
     * [ListItem].
     *
     * @param containerColor the container color of this list item when enabled.
     * @param headlineColor the headline text content color of this list item when enabled.
     * @param leadingIconColor the color of this list item's leading content when enabled.
     * @param trailingIconColor the color of this list item's trailing content when enabled.
     * @param disabledHeadlineColor the content color of this list item when not enabled.
     * @param disabledLeadingIconColor the color of this list item's leading content when not
     *   enabled.
     * @param disabledTrailingIconColor the color of this list item's trailing content when not
     *   enabled.
     */
    @Composable
    fun colors(
        containerColor: Color = MaterialTheme.colorScheme.surface,
        headlineColor: Color = MaterialTheme.colorScheme.onSurface,
        leadingIconColor: Color = MaterialTheme.colorScheme.onSurface,
        trailingIconColor: Color = MaterialTheme.colorScheme.onSurface,
        selectedHeadlineColor: Color = MaterialTheme.colorScheme.tertiary,
        disabledHeadlineColor: Color =
            headlineColor.copy(alpha = RelayListTokens.RelayListItemDisabledLabelTextOpacity),
        disabledLeadingIconColor: Color =
            leadingIconColor.copy(alpha = RelayListTokens.RelayListItemDisabledLeadingIconOpacity),
        disabledTrailingIconColor: Color =
            trailingIconColor.copy(alpha = RelayListTokens.RelayListItemDisabledTrailingIconOpacity),
    ): RelayListItemColors =
        RelayListItemColors(
            containerColor = containerColor,
            headlineColor = headlineColor,
            leadingIconColor = leadingIconColor,
            trailingIconColor = trailingIconColor,
            selectedHeadlineColor = selectedHeadlineColor,
            disabledHeadlineColor = disabledHeadlineColor,
            disabledLeadingIconColor = disabledLeadingIconColor,
            disabledTrailingIconColor = disabledTrailingIconColor,
        )
}

object RelayListTokens {
    const val RelayListItemDisabledLabelTextOpacity = 0.38f
    const val RelayListItemDisabledLeadingIconOpacity = 0.38f
    const val RelayListItemDisabledTrailingIconOpacity = 0.38f
}

@Preview(backgroundColor = 0xFFFFFFFF, showBackground = true)
@PreviewFontScale
@Composable
private fun PreviewRelayListItem() {
    AppTheme {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            RelayListItem(
                modifier = Modifier.fillMaxWidth(),
                content = {
                    Text(
                        "Hello world",
                        modifier = Modifier.padding(16.dp).padding(start = 58.dp).fillMaxSize(),
                    )
                },
            )
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
                leadingContent = {
                    Box(
                        modifier =
                            Modifier.fillMaxSize().clickable(onClick = { /* Handle click */ }),
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
            RelayListItem(
                modifier = Modifier.fillMaxWidth(),
                selected = true,
                content = {
                    Row(
                        modifier = Modifier.padding(16.dp).padding(start = 58.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Icon(imageVector = Icons.Default.Check, contentDescription = null)
                        Spacer(Modifier.width(8.dp))
                        Text(
                            "Hello world fsadhkuhfiuskahf iuhsadhuf sa",
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                },
                trailingContent = {
                    Box(
                        modifier =
                            Modifier.fillMaxSize().clickable(onClick = { /* Handle click */ }),
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
            RelayListItem(
                modifier = Modifier.fillMaxWidth(),
                content = {
                    Text(
                        "Hello world iuhsadhuf sa",
                        modifier =
                            Modifier.clickable { /* Handle click */ }.padding(16.dp).fillMaxSize(),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                },
                leadingContent = {
                    Box(
                        modifier =
                            Modifier.fillMaxSize().clickable(onClick = { /* Handle click */ }),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(
                            modifier = Modifier.padding(16.dp),
                            imageVector = Icons.Default.KeyboardArrowDown,
                            contentDescription = null,
                        )
                    }
                },
                trailingContent = {
                    Box(
                        modifier =
                            Modifier.fillMaxSize().clickable(onClick = { /* Handle click */ }),
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
}
