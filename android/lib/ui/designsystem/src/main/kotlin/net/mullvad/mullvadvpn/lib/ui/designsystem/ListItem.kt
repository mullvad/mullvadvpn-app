package net.mullvad.mullvadvpn.lib.ui.designsystem

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.sizeIn
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material.icons.rounded.KeyboardArrowDown
import androidx.compose.material.icons.rounded.Star
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewFontScale
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull

enum class Hierarchy {
    Parent,
    Child1,
    Child2,
    Child3,
}

enum class Position {
    Single,
    Top,
    Middle,
    Bottom,
}

enum class ListItemClickArea {
    All,
    LeadingAndMain,
}

data class CornerSize(val topStart: Dp, val topEnd: Dp, val bottomStart: Dp, val bottomEnd: Dp)

val Position.cornerSize: CornerSize
    get() =
        ListTokens.listItemRoundedCornerSize.let { size ->
            when (this) {
                Position.Single -> CornerSize(size, size, size, size)
                Position.Top -> CornerSize(size, size, 0.dp, 0.dp)
                Position.Bottom -> CornerSize(0.dp, 0.dp, size, size)
                Position.Middle -> CornerSize(0.dp, 0.dp, 0.dp, 0.dp)
            }
        }

val Hierarchy.paddingStart: Dp
    get() =
        when (this) {
            Hierarchy.Parent -> 0.dp
            Hierarchy.Child1 -> ListTokens.listItemPaddingStart
            Hierarchy.Child2 -> ListTokens.listItemPaddingStart * 2
            Hierarchy.Child3 -> ListTokens.listItemPaddingStart * 3
        }

fun Hierarchy.nextDown(): Hierarchy =
    when (this) {
        Hierarchy.Parent -> Hierarchy.Child1
        Hierarchy.Child1 -> Hierarchy.Child2
        Hierarchy.Child2 -> Hierarchy.Child3
        Hierarchy.Child3 -> error("Child3 is the lowest hierarchy")
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

object RelayListTokens {
    const val RelayListItemDisabledLabelTextOpacity = AlphaInactive
}

@Composable
@Suppress("LongMethod")
fun MullvadListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    colors: ListItemColors = ListItemDefaults.colors(),
    backgroundAlpha: Float = 1f,
    isEnabled: Boolean = true,
    isSelected: Boolean = false,
    testTag: String? = null,
    mainClickArea: ListItemClickArea = ListItemClickArea.All,
    onClick: (() -> Unit)? = null,
    onLongClick: (() -> Unit)? = null,
    leadingContent: @Composable (BoxScope.() -> Unit)? = null,
    trailingContent: @Composable (BoxScope.() -> Unit)? = null,
    content: @Composable (BoxScope.() -> Unit),
) {
    val size = position.cornerSize
    val cornerTopStart = animateDpAsState(targetValue = size.topStart)
    val cornerTopEnd = animateDpAsState(targetValue = size.topEnd)
    val cornerBottomStart = animateDpAsState(targetValue = size.bottomStart)
    val cornerBottomEnd = animateDpAsState(targetValue = size.bottomEnd)

    Surface(
        modifier =
            modifier
                .defaultMinSize(minHeight = ListTokens.listItemMinHeight)
                .height(IntrinsicSize.Min),
        shape =
            RoundedCornerShape(
                topStart = cornerTopStart.value,
                topEnd = cornerTopEnd.value,
                bottomStart = cornerBottomStart.value,
                bottomEnd = cornerBottomEnd.value,
            ),
    ) {
        Row(
            modifier =
                Modifier.background(colors.containerColor(hierarchy).copy(alpha = backgroundAlpha))
                    .applyIfNotNull(testTag) { testTag(it) }
                    .applyIfNotNull(onClick, and = mainClickArea == ListItemClickArea.All) {
                        combinedClickable(
                            enabled = isEnabled,
                            onClick = it,
                            onLongClick = onLongClick,
                        )
                    }
                    .semantics { selected = isSelected },
            verticalAlignment = Alignment.CenterVertically,
        ) {
            // This row is needed to prevent the main click ripple from travelling over
            // the trailing content when that shouldn't happen.
            Row(
                modifier =
                    Modifier.weight(1f)
                        .applyIfNotNull(
                            onClick,
                            and = mainClickArea == ListItemClickArea.LeadingAndMain,
                        ) {
                            combinedClickable(
                                enabled = isEnabled,
                                onClick = it,
                                onLongClick = onLongClick,
                            )
                        }
                        .padding(start = ListTokens.listItemPaddingStart + hierarchy.paddingStart)
            ) {
                if (leadingContent != null) {
                    Box(
                        modifier = Modifier.fillMaxHeight(),
                        contentAlignment = Alignment.CenterStart,
                    ) {
                        ProvideContentColorTextStyle(
                            colors.headlineColor(isEnabled, isSelected),
                            MaterialTheme.typography.titleMedium,
                        ) {
                            leadingContent(this)
                        }
                    }
                }

                Box(modifier = Modifier.fillMaxHeight(), contentAlignment = Alignment.CenterStart) {
                    ProvideContentColorTextStyle(
                        colors.headlineColor(isEnabled, isSelected),
                        MaterialTheme.typography.titleMedium,
                    ) {
                        content(this)
                    }
                }
            }

            if (trailingContent != null) {
                Box(
                    modifier =
                        Modifier.sizeIn(minWidth = ListTokens.listItemButtonWidth)
                            .width(IntrinsicSize.Max)
                            .fillMaxHeight(),
                    contentAlignment = Alignment.Center,
                ) {
                    ProvideContentColorTextStyle(
                        colors.trailingIconColor,
                        MaterialTheme.typography.titleMedium,
                    ) {
                        trailingContent(this)
                    }
                }
            }
        }
    }
}

// Based of ListItem
@Immutable
class ListItemColors(
    val containerColorParent: Color,
    val containerColorChild1: Color,
    val containerColorChild2: Color,
    val containerColorChild3: Color,
    val headlineColor: Color,
    val trailingIconColor: Color,
    val selectedHeadlineColor: Color,
    val disabledHeadlineColor: Color,
) {
    @Stable
    internal fun headlineColor(enabled: Boolean, selected: Boolean): Color =
        when {
            !enabled -> disabledHeadlineColor
            selected -> selectedHeadlineColor
            else -> headlineColor
        }

    internal fun containerColor(hierarchy: Hierarchy) =
        when (hierarchy) {
            // Using primary is a workaround to ensure enough contrast between lowest depth (3) and
            // the background.
            Hierarchy.Parent -> containerColorParent
            Hierarchy.Child1 -> containerColorChild1
            Hierarchy.Child2 -> containerColorChild2
            Hierarchy.Child3 -> containerColorChild3
        }
}

object ListItemDefaults {
    @Composable
    fun colors(
        containerColorParent: Color = MaterialTheme.colorScheme.primary,
        containerColorChild1: Color = MaterialTheme.colorScheme.surfaceContainerHighest,
        containerColorChild2: Color = MaterialTheme.colorScheme.surfaceContainerHigh,
        containerColorChild3: Color = MaterialTheme.colorScheme.surfaceContainerLow,
        headlineColor: Color = MaterialTheme.colorScheme.onSurface,
        trailingIconColor: Color = MaterialTheme.colorScheme.onSurface,
        selectedHeadlineColor: Color = MaterialTheme.colorScheme.tertiary,
        disabledHeadlineColor: Color =
            headlineColor.copy(alpha = ListTokens.ListItemDisabledLabelTextOpacity),
    ): ListItemColors =
        ListItemColors(
            containerColorParent = containerColorParent,
            containerColorChild1 = containerColorChild1,
            containerColorChild2 = containerColorChild2,
            containerColorChild3 = containerColorChild3,
            headlineColor = headlineColor,
            trailingIconColor = trailingIconColor,
            selectedHeadlineColor = selectedHeadlineColor,
            disabledHeadlineColor = disabledHeadlineColor,
        )
}

object ListTokens {
    const val ListItemDisabledLabelTextOpacity = AlphaInactive

    val listItemMinHeight = 56.dp
    val listItemButtonWidth = 56.dp
    val listItemPaddingStart = 16.dp
    val listItemRoundedCornerSize = 16.dp
}

@Preview
@PreviewFontScale
@Composable
private fun PreviewLeadingContentListItem() {
    AppTheme {
        MullvadListItem(
            modifier = Modifier.fillMaxWidth(),
            hierarchy = Hierarchy.Child3,
            isEnabled = false,
            isSelected = true,
            leadingContent = {
                Icon(
                    modifier = Modifier.size(24.dp).align(Alignment.Center),
                    imageVector = Icons.Rounded.Star,
                    contentDescription = null,
                )
            },
            content = {
                Text(
                    "Hello world fsadhkuhfiuskahf iuhsadhuf sa",
                    modifier =
                        Modifier.padding(start = 4.dp, top = 16.dp, bottom = 16.dp)
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
                        imageVector = Icons.Rounded.KeyboardArrowDown,
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
private fun PreviewLeadingListItem() {
    AppTheme {
        MullvadListItem(
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
                        imageVector = Icons.Rounded.KeyboardArrowDown,
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
private fun PreviewTrailingListItem() {
    AppTheme {
        MullvadListItem(
            modifier = Modifier.fillMaxWidth(),
            isSelected = true,
            content = {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(imageVector = Icons.Rounded.Check, contentDescription = null)
                    Spacer(Modifier.width(8.dp))
                    Text("Sample Item", maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            },
            trailingContent = {
                Box(
                    modifier = Modifier.fillMaxSize().clickable(onClick = {}),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        modifier = Modifier.padding(16.dp),
                        imageVector = Icons.Rounded.Add,
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
private fun PreviewHierarchyListItem() {
    AppTheme {
        MullvadListItem(
            modifier = Modifier.fillMaxWidth(),
            isSelected = true,
            hierarchy = Hierarchy.Child3,
            content = {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(imageVector = Icons.Rounded.Check, contentDescription = null)
                    Spacer(Modifier.width(8.dp))
                    Text("Sample Item", maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            },
            trailingContent = {
                Box(
                    modifier = Modifier.fillMaxSize().clickable(onClick = {}),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        modifier = Modifier.padding(16.dp),
                        imageVector = Icons.Rounded.Add,
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
private fun PreviewListItemPositions() {
    AppTheme {
        MullvadListItem(
            modifier = Modifier.fillMaxWidth(),
            position = Position.Top,
            content = {
                Row(
                    modifier = Modifier.padding(16.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("Sample Item", maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            },
            trailingContent = {
                Box(
                    modifier = Modifier.fillMaxSize().clickable(onClick = {}),
                    contentAlignment = Alignment.Center,
                ) {
                    Icon(
                        modifier = Modifier.padding(16.dp),
                        imageVector = Icons.Rounded.Add,
                        contentDescription = null,
                    )
                }
            },
        )
    }
}
