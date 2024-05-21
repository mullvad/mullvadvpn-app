package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.Dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.Chevron
import net.mullvad.mullvadvpn.compose.component.MullvadCheckbox
import net.mullvad.mullvadvpn.compose.preview.RelayItemCheckableCellPreviewParameterProvider
import net.mullvad.mullvadvpn.compose.preview.RelayItemStatusCellPreviewParameterProvider
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.onSelected
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.relaylist.children

@Composable
@Preview
private fun PreviewStatusRelayLocationCell(
    @PreviewParameter(RelayItemStatusCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.background)) {
            relayItems.map { StatusRelayLocationCell(relay = it) }
        }
    }
}

@Composable
@Preview
private fun PreviewCheckableRelayLocationCell(
    @PreviewParameter(RelayItemCheckableCellPreviewParameterProvider::class)
    relayItems: List<RelayItem.Location.Country>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.background)) {
            relayItems.map { CheckableRelayLocationCell(relay = it) }
        }
    }
}

@Composable
fun StatusRelayLocationCell(
    relay: RelayItem,
    modifier: Modifier = Modifier,
    activeColor: Color = MaterialTheme.colorScheme.selected,
    inactiveColor: Color = MaterialTheme.colorScheme.error,
    disabledColor: Color = MaterialTheme.colorScheme.onSecondary,
    selectedItem: RelayItemId? = null,
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onLongClick: (item: RelayItem) -> Unit = {},
) {
    RelayLocationCell(
        relay = relay,
        leadingContent = { relayItem ->
            val selected = selectedItem == relayItem.id
            Box(
                modifier =
                    Modifier.align(Alignment.CenterStart)
                        .size(Dimens.relayCircleSize)
                        .background(
                            color =
                                when {
                                    selected -> Color.Unspecified
                                    relayItem is RelayItem.CustomList && !relayItem.hasChildren ->
                                        disabledColor
                                    relayItem.active -> activeColor
                                    else -> inactiveColor
                                },
                            shape = CircleShape
                        )
            )
            Image(
                painter = painterResource(id = R.drawable.icon_tick),
                modifier =
                    Modifier.align(Alignment.CenterStart)
                        .alpha(
                            if (selected) {
                                AlphaVisible
                            } else {
                                AlphaInvisible
                            }
                        ),
                contentDescription = null
            )
        },
        modifier = modifier,
        specialBackgroundColor = { relayItem ->
            when {
                selectedItem == relayItem.id -> MaterialTheme.colorScheme.selected
                relayItem is RelayItem.CustomList && !relayItem.active ->
                    MaterialTheme.colorScheme.surfaceTint
                else -> null
            }
        },
        onClick = onSelectRelay,
        onLongClick = onLongClick,
        depth = 0
    )
}

@Composable
fun CheckableRelayLocationCell(
    relay: RelayItem,
    modifier: Modifier = Modifier,
    onRelayCheckedChange: (item: RelayItem, isChecked: Boolean) -> Unit = { _, _ -> },
    selectedRelays: Set<RelayItem> = emptySet(),
) {
    RelayLocationCell(
        relay = relay,
        leadingContent = { relayItem ->
            val checked = selectedRelays.contains(relayItem)
            MullvadCheckbox(
                checked = checked,
                onCheckedChange = { isChecked -> onRelayCheckedChange(relayItem, isChecked) }
            )
        },
        leadingContentStartPadding = Dimens.cellStartPaddingInteractive,
        modifier = modifier,
        onClick = { onRelayCheckedChange(it, !selectedRelays.contains(it)) },
        onLongClick = null,
        depth = 0
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun RelayLocationCell(
    relay: RelayItem,
    leadingContent: @Composable BoxScope.(relay: RelayItem) -> Unit,
    modifier: Modifier = Modifier,
    leadingContentStartPadding: Dp = Dimens.cellStartPadding,
    leadingContentStarPaddingModifier: Dp = Dimens.mediumPadding,
    specialBackgroundColor: @Composable (relayItem: RelayItem) -> Color? = { null },
    onClick: (item: RelayItem) -> Unit,
    onLongClick: ((item: RelayItem) -> Unit)?,
    depth: Int
) {
    val startPadding = leadingContentStartPadding + leadingContentStarPaddingModifier * depth
    val expanded =
        rememberSaveable(key = relay.expanded.toString()) { mutableStateOf(relay.expanded) }
    Column(
        modifier =
            modifier
                .fillMaxWidth()
                .padding(top = Dimens.listItemDivider)
                .wrapContentHeight()
                .fillMaxWidth()
    ) {
        Row(
            modifier =
                Modifier.align(Alignment.Start)
                    .wrapContentHeight()
                    .height(IntrinsicSize.Min)
                    .fillMaxWidth()
                    .background(specialBackgroundColor.invoke(relay) ?: depth.toBackgroundColor())
        ) {
            Row(
                modifier =
                    Modifier.weight(1f)
                        .combinedClickable(
                            enabled = relay.active,
                            onClick = { onClick(relay) },
                            onLongClick = { onLongClick?.invoke(relay) },
                        )
            ) {
                Box(
                    modifier =
                        Modifier.align(Alignment.CenterVertically).padding(start = startPadding)
                ) {
                    leadingContent(relay)
                }
                Name(
                    modifier = Modifier.weight(1f).align(Alignment.CenterVertically),
                    relay = relay
                )
            }
            if (relay.hasChildren) {
                ExpandButton(isExpanded = expanded.value) { expand -> expanded.value = expand }
            }
        }
        if (expanded.value) {
            relay.children().forEach {
                RelayLocationCell(
                    relay = it,
                    onClick = onClick,
                    modifier = Modifier.animateContentSize(),
                    leadingContent = leadingContent,
                    specialBackgroundColor = specialBackgroundColor,
                    onLongClick = onLongClick,
                    depth = depth + 1,
                )
            }
        }
    }
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
        3 -> MaterialTheme.colorScheme.surfaceContainerLowest
        else -> MaterialTheme.colorScheme.surfaceContainerLowest
    }
