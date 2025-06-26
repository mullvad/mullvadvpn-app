package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ExpandChevron
import net.mullvad.mullvadvpn.compose.component.MullvadCheckbox
import net.mullvad.mullvadvpn.compose.preview.RelayItemCheckableCellPreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayListItemState
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOCATION_CELL_TEST_TAG

@Composable
@Preview
private fun PreviewCheckableRelayLocationCell(
    @PreviewParameter(RelayItemCheckableCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.map {
                CheckableRelayLocationCell(
                    item = it,
                    checked = false,
                    expanded = false,
                    depth = 0,
                    onExpand = {},
                    modifier = Modifier.testTag(LOCATION_CELL_TEST_TAG),
                )
            }
        }
    }
}

@Composable
fun StatusRelayItemCell(
    item: RelayItem,
    isSelected: Boolean,
    state: RelayListItemState?,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit),
    isExpanded: Boolean = false,
    depth: Int = 0,
) {
    RelayItemCell1(
        modifier = modifier,
        item = item,
        isSelected = isSelected,
        state = state,
        onClick = onClick,
        onLongClick = onLongClick,
        onToggleExpand = onToggleExpand,
        isExpanded = isExpanded,
        depth = depth,
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun RelayItemCell1(
    modifier: Modifier = Modifier,
    item: RelayItem,
    isSelected: Boolean,
    state: RelayListItemState?,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: (Boolean) -> Unit,
    isExpanded: Boolean,
    depth: Int,
    content: @Composable (RowScope.() -> Unit)? = null,
) {
    RelayListItem(
        modifier = modifier,
        selected = isSelected,
        content = {
            Row(
                modifier =
                    Modifier.let { if (!item.hasChildren) it.padding(start = 58.dp) else it }
                        .padding(16.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (isSelected) {
                    Icon(
                        imageVector = Icons.Default.Check,
                        contentDescription = null,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                    )
                }

                Name(name = item.name, state = state, active = item.active)
            }
        },
        leadingContent =
            if (item.hasChildren) {
                {
                    ExpandChevron(
                        color = MaterialTheme.colorScheme.onSurface,
                        isExpanded = isExpanded,
                        modifier =
                            Modifier.clickable { onToggleExpand(!isExpanded) }
                                .fillMaxSize()
                                .padding(16.dp)
                                .testTag(EXPAND_BUTTON_TEST_TAG),
                    )
                }
            } else {
                null
            },
        onClick = onClick,
        onLongClick = onLongClick,
        trailingContent = {
            Icon(
                modifier =
                    Modifier.clickable(enabled = true, onClick = onLongClick ?: {})
                        .fillMaxSize()
                        .padding(16.dp),
                imageVector = Icons.Default.Add,
                contentDescription = null,
            )
        },
        colors = RelayListItemDefaults.colors(containerColor = depth.toBackgroundColor()),
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun RelayItemCell(
    modifier: Modifier = Modifier,
    item: RelayItem,
    isSelected: Boolean,
    state: RelayListItemState?,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: (Boolean) -> Unit,
    isExpanded: Boolean,
    depth: Int,
    content: @Composable (RowScope.() -> Unit)? = null,
) {

    val leadingContentStartPadding = Dimens.cellStartPadding
    val leadingContentStarPaddingModifier = Dimens.mediumPadding
    val startPadding = leadingContentStartPadding + leadingContentStarPaddingModifier * depth
    Row(
        modifier =
            modifier
                .fillMaxWidth()
                .height(IntrinsicSize.Min)
                .background(
                    when {
                        isSelected -> MaterialTheme.colorScheme.selected
                        else -> depth.toBackgroundColor()
                    }
                ),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        // Duplicate row is needed for selection of the item on TV.
        Row(
            modifier =
                Modifier.combinedClickable(
                        enabled = state == null && item.active,
                        onClick = onClick,
                        onLongClick = onLongClick,
                    )
                    .padding(start = startPadding)
                    .weight(1f),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (content != null) {
                content()
            }
            Name(name = item.name, state = state, active = item.active)
        }

        if (item.hasChildren) {
            ExpandButton(
                color = MaterialTheme.colorScheme.onSurface,
                isExpanded = isExpanded,
                onClick = { onToggleExpand(!isExpanded) },
                modifier = Modifier.testTag(EXPAND_BUTTON_TEST_TAG),
            )
        }
    }
}

@Composable
fun CheckableRelayLocationCell(
    item: RelayItem,
    modifier: Modifier = Modifier,
    checked: Boolean,
    onRelayCheckedChange: (isChecked: Boolean) -> Unit = { _ -> },
    expanded: Boolean,
    onExpand: (Boolean) -> Unit,
    depth: Int,
) {
    RelayItemCell(
        modifier = modifier,
        item = item,
        isSelected = false,
        state = null,
        onClick = { onRelayCheckedChange(!checked) },
        onToggleExpand = onExpand,
        isExpanded = expanded,
        depth = depth,
        content = {
            MullvadCheckbox(
                checked = checked,
                onCheckedChange = { isChecked -> onRelayCheckedChange(isChecked) },
            )
        },
    )
}

@Composable
private fun Name(
    modifier: Modifier = Modifier,
    name: String,
    state: RelayListItemState?,
    active: Boolean,
) {
    Text(
        text = state?.let { name.withSuffix(state) } ?: name,
        modifier =
            modifier.alpha(
                if (state == null && active) {
                    AlphaVisible
                } else {
                    AlphaInactive
                }
            ),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun ExpandButton(
    modifier: Modifier,
    color: Color,
    isExpanded: Boolean,
    onClick: (expand: Boolean) -> Unit,
) {
    VerticalDivider(
        color = MaterialTheme.colorScheme.surface,
        modifier = Modifier.padding(vertical = Dimens.verticalDividerPadding),
    )
    ExpandChevron(
        color = color,
        isExpanded = isExpanded,
        modifier =
            modifier
                .fillMaxHeight()
                .clickable { onClick(!isExpanded) }
                .padding(horizontal = Dimens.largePadding),
    )
}

@Suppress("MagicNumber")
@Composable
private fun Int.toBackgroundColor(): Color =
    when (this) {
        0 -> MaterialTheme.colorScheme.surfaceContainerHighest
        1 -> MaterialTheme.colorScheme.surfaceContainerHigh
        2 -> MaterialTheme.colorScheme.surfaceContainerLow
        else -> MaterialTheme.colorScheme.surfaceContainerLowest
    }

@Composable
private fun String.withSuffix(state: RelayListItemState) =
    when (state) {
        RelayListItemState.USED_AS_EXIT -> stringResource(R.string.x_exit, this)
        RelayListItemState.USED_AS_ENTRY -> stringResource(R.string.x_entry, this)
    }
