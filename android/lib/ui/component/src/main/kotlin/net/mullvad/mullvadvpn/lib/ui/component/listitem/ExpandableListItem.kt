package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Delete
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.lib.ui.component.button.ListItemIconButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListTokens
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull

@Preview
@Composable
private fun PreviewExpandedEnabledExpandableListItem() {
    AppTheme {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            ExpandableListItem(
                title = "Expandable row title",
                isExpanded = true,
                isEnabled = true,
                onCellClicked = {},
                onInfoClicked = {},
            )
            ExpandableListItem(
                title = "Expandable row with content",
                isExpanded = true,
                isEnabled = true,
                onCellClicked = {},
                expandedContent = {
                    Box(
                        modifier =
                            Modifier.fillMaxWidth()
                                .height(50.dp)
                                .background(MaterialTheme.colorScheme.surfaceContainerHighest),
                    )
                },
            )
            ExpandableListItem(
                title = "Expandable row with content",
                isExpanded = true,
                isEnabled = true,
                onCellClicked = {},
                onInfoClicked = {},
                extraTrailingContent = {
                    ListItemIconButton(onClick = {}) {
                        Icon(imageVector = Icons.Rounded.Delete, contentDescription = null)
                    }
                },
            )
        }
    }
}

@Composable
fun ExpandableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    title: String,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    singeLine: Boolean = true,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
    expandedContent: @Composable (() -> Unit),
    extraTrailingContent: @Composable ((RowScope) -> Unit)? = null,
) {

    Column(modifier = modifier) {
        ExpandableListItem(
            hierarchy = hierarchy,
            position = if (isExpanded) Position.Top else Position.Single,
            isExpanded = isExpanded,
            isEnabled = isEnabled,
            onCellClicked = onCellClicked,
            onInfoClicked = onInfoClicked,
            extraTrailingContent = extraTrailingContent,
            content = {
                Text(
                    text = title,
                    maxLines = if (singeLine) 1 else Int.MAX_VALUE,
                    overflow = TextOverflow.Ellipsis,
                )
            },
        )
        AnimatedVisibility(visible = isExpanded) {
            Surface(
                shape =
                    RoundedCornerShape(
                        topStart = 0.dp,
                        topEnd = 0.dp,
                        bottomStart = ListTokens.listItemRoundedCornerSize,
                        bottomEnd = ListTokens.listItemRoundedCornerSize,
                    ),
            ) {
                expandedContent()
            }
        }
    }
}

@Composable
fun ExpandableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    singeLine: Boolean = true,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
    extraTrailingContent: @Composable ((RowScope) -> Unit)? = null,
) {
    ExpandableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isExpanded = isExpanded,
        isEnabled = isEnabled,
        onCellClicked = onCellClicked,
        onInfoClicked = onInfoClicked,
        extraTrailingContent = extraTrailingContent,
        content = {
            Text(
                text = title,
                maxLines = if (singeLine) 1 else Int.MAX_VALUE,
                overflow = TextOverflow.Ellipsis,
            )
        },
    )
}

@Composable
fun ExpandableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    isExpanded: Boolean,
    isEnabled: Boolean = true,
    backgroundAlpha: Float = 1f,
    onCellClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
    content: @Composable (BoxScope) -> Unit,
    extraTrailingContent: @Composable ((RowScope) -> Unit)? = null,
) {
    MullvadListItem(
        modifier = modifier.applyIfNotNull(onInfoClicked) { focusProperties { canFocus = false } },
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        onClick = { onCellClicked(!isExpanded) },
        backgroundAlpha = backgroundAlpha,
        content = content,
        trailingContent = {
            Row(modifier = modifier.fillMaxSize()) {
                extraTrailingContent?.invoke(this)
                if (onInfoClicked != null) {
                    ListItemIconButton(onClick = onInfoClicked) {
                        Icon(imageVector = Icons.Rounded.Info, contentDescription = null)
                    }
                }
                ListItemIconButton(
                    modifier =
                        Modifier.padding(end = ChevronIconPaddingEnd)
                            .testTag(EXPAND_BUTTON_TEST_TAG),
                    containerWidth = ChevronContainerWidth,
                    onClick = { onCellClicked(!isExpanded) },
                ) {
                    ExpandChevron(isExpanded = isExpanded)
                }
            }
        },
    )
}

private val ChevronContainerWidth = 60.dp
private val ChevronIconPaddingEnd = 8.dp
