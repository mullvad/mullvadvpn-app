package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevronDivider
import net.mullvad.mullvadvpn.lib.ui.component.listitem.LeadingContentAnimatedVisibility
import net.mullvad.mullvadvpn.lib.ui.component.toAnnotatedString
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemClickArea
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.icon.MultihopWhenNeeded
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_LIST_ENTRY_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_LIST_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.GEOLOCATION_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.RECENT_ITEM_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive

@Composable
@Preview
private fun PreviewSelectableRelayLocationItem(
    @PreviewParameter(SelectableRelayListItemPreviewParameterProvider::class)
    relayItems: List<RelayListItem.SelectableItem>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.forEach {
                Spacer(Modifier.size(1.dp))
                SelectableRelayListItem(
                    relayListItem = it,
                    onClick = {},
                    onToggleExpand = {},
                    showMultihopWhenNeededIcon = it.item is RelayItem.Location.Relay,
                )
            }
        }
    }
}

@Composable
fun SelectableRelayListItem(
    modifier: Modifier = Modifier,
    relayListItem: RelayListItem.SelectableItem,
    annotatedTitle: AnnotatedString? = null,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit),
    showMultihopWhenNeededIcon: Boolean = false,
) {
    val active = relayListItem.item.active
    val selected = relayListItem.isSelected

    val colors = ListItemDefaults.colors()

    MullvadListItem(
        modifier = modifier,
        hierarchy = relayListItem.hierarchy,
        position = relayListItem.itemPosition,
        isSelected = selected,
        isEnabled = true,
        onClick = onClick,
        onLongClick = onLongClick,
        colors = colors,
        testTag =
            when (relayListItem) {
                is RelayListItem.CustomListEntryItem -> CUSTOM_LIST_ENTRY_ITEM_TAG
                is RelayListItem.CustomListItem -> CUSTOM_LIST_ITEM_TAG
                is RelayListItem.GeoLocationItem -> GEOLOCATION_ITEM_TAG
                is RelayListItem.RecentListItem -> RECENT_ITEM_TAG
            },
        mainClickArea =
            if (relayListItem.canExpand) ListItemClickArea.LeadingAndMain
            else ListItemClickArea.All,
        leadingContent = {
            LeadingContentAnimatedVisibility(
                modifier = Modifier.align(Alignment.Center),
                visible = selected || !active,
            ) {
                if (selected) {
                    Icon(
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        imageVector = Icons.Rounded.Check,
                        contentDescription = null,
                        tint =
                            if (!active) MaterialTheme.colorScheme.error
                            else LocalContentColor.current,
                    )
                } else if (!active) {
                    InactiveRelayIndicator(
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        tint = MaterialTheme.colorScheme.error,
                    )
                }
            }
        },
        content = {
            Name(
                name = annotatedTitle ?: relayListItem.item.name.toAnnotatedString(),
                state = relayListItem.state,
                colors.headlineColor(enabled = active, selected = selected),
            )
        },
        trailingContent =
            when {
                showMultihopWhenNeededIcon -> {
                    {
                        Icon(
                            modifier = Modifier.width(Dimens.dividerButtonWidth),
                            imageVector = MultihopWhenNeeded,
                            tint =
                                if (selected) MaterialTheme.colorScheme.positive
                                else MaterialTheme.colorScheme.onSurface,
                            contentDescription = stringResource(R.string.multihop_when_needed),
                        )
                    }
                }
                relayListItem.canExpand -> {
                    {
                        ExpandChevronDivider(
                            isExpanded = relayListItem.expanded,
                            modifier = Modifier.testTag(EXPAND_BUTTON_TEST_TAG),
                            onClick = { onToggleExpand(!relayListItem.expanded) },
                        )
                    }
                }
                else -> null
            },
    )
}

@Composable
internal fun Name(name: AnnotatedString, state: RelayListItemState?, textColor: Color) {
    Text(
        text = state?.let { name.withSuffix(state) } ?: name,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
        color = textColor,
    )
}

@Composable
private fun AnnotatedString.withSuffix2(state: RelayListItemState) =
    when (state) {
        RelayListItemState.USED_AS_EXIT -> stringResource(R.string.x_exit, this)
        RelayListItemState.USED_AS_ENTRY -> stringResource(R.string.x_entry, this)
    }

@Composable
private fun AnnotatedString.withSuffix(state: RelayListItemState): AnnotatedString {
    val resId = when (state) {
        RelayListItemState.USED_AS_EXIT -> R.string.x_exit
        RelayListItemState.USED_AS_ENTRY -> R.string.x_entry
    }

    val template = stringResource(resId)

    val placeholder = when {
        template.contains("%1\$s") -> "%1\$s"
        template.contains("%s") -> "%s"
        else -> null
    }

    // We need to do this instead of using stringResource to prevent stripping the annotations.
    return buildAnnotatedString {
        if (placeholder != null) {
            val parts = template.split(placeholder, limit = 2)
            append(parts[0])
            append(this@withSuffix)
            append(parts[1])
        } else {
            append(this@withSuffix)
        }
    }
}

@Composable
fun InactiveRelayIndicator(modifier: Modifier = Modifier, tint: Color) {
    Box(
        modifier =
            modifier
                .size(Dimens.listIconSize)
                .padding(Dimens.relayCirclePadding)
                .background(color = tint, shape = CircleShape)
    )
}
