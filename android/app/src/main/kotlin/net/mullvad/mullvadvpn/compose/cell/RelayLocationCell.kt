package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.Chevron
import net.mullvad.mullvadvpn.compose.component.MullvadCheckbox
import net.mullvad.mullvadvpn.compose.preview.RelayItemCheckableCellPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.selected

@Composable
@Preview
private fun PreviewCheckableRelayLocationCell(
    @PreviewParameter(RelayItemCheckableCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.background)) {
            relayItems.map {
                CheckableRelayLocationCell(
                    item = it,
                    checked = false,
                    expanded = false,
                    depth = 0,
                    onExpand = {}
                )
            }
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun StatusRelayItemCell(
    item: RelayItem,
    isSelected: Boolean,
    modifier: Modifier = Modifier,
    onClick: () -> Unit = {},
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit) = {},
    isExpanded: Boolean = false,
    depth: Int = 0,
    activeColor: Color = MaterialTheme.colorScheme.selected,
    inactiveColor: Color = MaterialTheme.colorScheme.error,
    disabledColor: Color = MaterialTheme.colorScheme.onSecondary,
) {

    RelayItemCell(
        modifier = modifier,
        item,
        isSelected,
        leadingContent = {
            if (isSelected) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_tick),
                    contentDescription = null
                )
            } else {
                Box(
                    modifier =
                        Modifier.padding(4.dp)
                            .size(Dimens.relayCircleSize)
                            .background(
                                color =
                                    when {
                                        isSelected -> Color.Unspecified
                                        item is RelayItem.CustomList && item.locations.isEmpty() ->
                                            disabledColor
                                        item.active -> activeColor
                                        else -> inactiveColor
                                    },
                                shape = CircleShape
                            )
                )
            }
        },
        onClick = onClick,
        onLongClick = onLongClick,
        onToggleExpand = onToggleExpand,
        isExpanded = isExpanded,
        depth = depth,
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun RelayItemCell(
    modifier: Modifier = Modifier,
    item: RelayItem,
    isSelected: Boolean,
    leadingContent: (@Composable RowScope.() -> Unit)? = null,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit),
    isExpanded: Boolean,
    depth: Int,
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
                        item is RelayItem.CustomList && !item.active ->
                            MaterialTheme.colorScheme.surfaceTint
                        else -> depth.toBackgroundColor()
                    }
                )
                .combinedClickable(
                    enabled = item.active,
                    onClick = onClick,
                    onLongClick = onLongClick,
                )
                .padding(start = startPadding),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (leadingContent != null) {
            leadingContent()
        }
        Name(modifier = Modifier.weight(1f), relay = item)

        if (item.hasChildren) {
            ExpandButton(isExpanded = isExpanded, onClick = { onToggleExpand(!isExpanded) })
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
    depth: Int
) {
    RelayItemCell(
        modifier = modifier,
        item = item,
        isSelected = false,
        leadingContent = {
            MullvadCheckbox(
                checked = checked,
                onCheckedChange = { isChecked -> onRelayCheckedChange(isChecked) }
            )
        },
        onClick = { onRelayCheckedChange(!checked) },
        onToggleExpand = onExpand,
        isExpanded = expanded,
        depth = depth
    )
}

@Composable
private fun Name(modifier: Modifier = Modifier, relay: RelayItem) {
    Text(
        text = relay.name,
        color = MaterialTheme.colorScheme.onSurface,
        modifier =
            modifier
                .alpha(
                    if (relay.active) {
                        AlphaVisible
                    } else {
                        AlphaInactive
                    }
                )
                .padding(horizontal = Dimens.smallPadding, vertical = Dimens.mediumPadding),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis
    )
}

@Composable
private fun RowScope.ExpandButton(isExpanded: Boolean, onClick: (expand: Boolean) -> Unit) {
    VerticalDivider(
        color = MaterialTheme.colorScheme.background,
        modifier = Modifier.padding(vertical = Dimens.verticalDividerPadding)
    )
    Chevron(
        isExpanded = isExpanded,
        modifier =
            Modifier.fillMaxHeight()
                .clickable { onClick(!isExpanded) }
                .padding(horizontal = Dimens.largePadding)
                .align(Alignment.CenterVertically)
    )
}

@Suppress("MagicNumber")
@Composable
private fun Int.toBackgroundColor(): Color =
    when (this) {
        0 -> MaterialTheme.colorScheme.surfaceContainerHigh
        1 -> MaterialTheme.colorScheme.surfaceContainerLow
        2 -> MaterialTheme.colorScheme.surfaceContainerLowest
        else -> MaterialTheme.colorScheme.surfaceContainerLowest
    }
