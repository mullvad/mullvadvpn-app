package net.mullvad.mullvadvpn.lib.ui.component.relaylist

import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.times
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.resource.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.theme.color.selected
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.RelayListTokens
import net.mullvad.mullvadvpn.lib.ui.tag.EXPAND_BUTTON_TEST_TAG

@Composable
@Preview
private fun PreviewSelectableRelayLocationItem(
    @PreviewParameter(SelectableRelayListItemPreviewParameterProvider::class)
    relayItems: List<RelayListItem.SelectableItem>
) {
    AppTheme {
        Column(Modifier.background(color = MaterialTheme.colorScheme.surface)) {
            relayItems.map {
                Spacer(Modifier.size(1.dp))
                SelectableRelayListItem(relayListItem = it, onClick = {}, onToggleExpand = {})
            }
        }
    }
}

@Composable
fun SelectableRelayListItem(
    relayListItem: RelayListItem.SelectableItem,
    modifier: Modifier = Modifier,
    onClick: () -> Unit,
    onLongClick: (() -> Unit)? = null,
    onToggleExpand: ((Boolean) -> Unit),
) {
    // TODO Fix colors
    val activeColor = MaterialTheme.colorScheme.selected
    val inactiveColor = MaterialTheme.colorScheme.error
    val disabledColor = MaterialTheme.colorScheme.onSurfaceVariant
    RelayListItem(
        modifier = modifier.clip(itemPosition = relayListItem.itemPosition),
        selected = relayListItem.isSelected,
        enabled = relayListItem.item.active,
        content = {
            Row(
                modifier =
                    Modifier.fillMaxSize()
                        .padding(start = relayListItem.depth * Dimens.mediumPadding)
                        .padding(Dimens.mediumPadding),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (relayListItem.isSelected) {
                    Icon(
                        imageVector = Icons.Default.Check,
                        contentDescription = null,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                    )
                } else if (!relayListItem.item.active) {
                    // TODO Fix design of this
                    Box(
                        modifier =
                            Modifier.padding(start = Dimens.smallPadding)
                                .size(Dimens.relayCircleSize)
                                .padding(2.dp)
                                .background(
                                    color =
                                        when {
                                            relayListItem.item is RelayItem.CustomList &&
                                                !relayListItem.item.hasChildren -> disabledColor

                                            relayListItem.state != null -> disabledColor
                                            relayListItem.item.active -> activeColor
                                            else -> inactiveColor
                                        },
                                    shape = CircleShape,
                                )
                    )
                }

                Name(
                    name = relayListItem.item.name,
                    state = relayListItem.state,
                    active = relayListItem.item.active,
                )
            }
        },
        onClick = onClick,
        onLongClick = onLongClick,
        trailingContent =
            if (relayListItem.item.hasChildren) {
                {
                    ExpandChevron(
                        isExpanded = relayListItem.expanded,
                        modifier =
                            Modifier.clickable { onToggleExpand(!relayListItem.expanded) }
                                .fillMaxSize()
                                .padding(Dimens.mediumPadding)
                                .testTag(EXPAND_BUTTON_TEST_TAG),
                    )
                }
            } else {
                null
            },
        colors =
            RelayListItemDefaults.colors(containerColor = relayListItem.depth.toBackgroundColor()),
    )
}

@Composable
internal fun Name(
    modifier: Modifier = Modifier,
    name: String,
    state: RelayListItemState?,
    active: Boolean,
) {
    Text(
        text = state?.let { name.withSuffix(state) } ?: name,
        style = MaterialTheme.typography.bodyLarge,
        modifier =
            modifier.alpha(
                if (state == null && active) {
                    AlphaVisible
                } else {
                    RelayListTokens.RelayListItemDisabledLabelTextOpacity
                }
            ),
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Suppress("MagicNumber")
@Composable
internal fun Int.toBackgroundColor(): Color =
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

@Composable
internal fun Modifier.clip(itemPosition: ItemPosition): Modifier {
    val topCornerSize =
        animateDpAsState(if (itemPosition.roundTop()) Dimens.relayItemCornerRadius else 0.dp)
    val bottomCornerSize =
        animateDpAsState(if (itemPosition.roundBottom()) Dimens.relayItemCornerRadius else 0.dp)
    return clip(
        RoundedCornerShape(
            topStart = CornerSize(topCornerSize.value),
            topEnd = CornerSize(topCornerSize.value),
            bottomStart = CornerSize(bottomCornerSize.value),
            bottomEnd = CornerSize(bottomCornerSize.value),
        )
    )
}
