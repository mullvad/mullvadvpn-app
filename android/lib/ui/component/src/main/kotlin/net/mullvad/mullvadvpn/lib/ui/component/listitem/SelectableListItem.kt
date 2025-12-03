package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandHorizontally
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

@Preview
@Composable
private fun PreviewSelectableListItem() {
    AppTheme {
        Column(
            Modifier.background(MaterialTheme.colorScheme.surface),
            verticalArrangement = Arrangement.spacedBy(Dimens.listItemDivider, Alignment.Bottom),
        ) {
            SelectableListItem(hierarchy = Hierarchy.Child1, title = "Selected", isSelected = true)
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                title = "Not Selected",
                isSelected = false,
            )
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                title = "Selected and disabled",
                isSelected = true,
                isEnabled = false,
            )
            SelectableListItem(
                hierarchy = Hierarchy.Child1,
                title = "Selected and disabled",
                subtitle = "Selected and disabled",
                isSelected = true,
                isEnabled = false,
            )
        }
    }
}

@Composable
fun SelectableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    isSelected: Boolean,
    isEnabled: Boolean = true,
    iconContentDescription: String? = null,
    onClick: (() -> Unit)? = null,
    testTag: String? = null,
    content: @Composable ((BoxScope.() -> Unit)),
    trailingContent: @Composable ((BoxScope.() -> Unit))? = null,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        isSelected = isSelected,
        testTag = testTag,
        onClick = onClick,
        leadingContent = {
            AnimatedVisibility(
                modifier = Modifier.align(Alignment.Center),
                visible = isSelected,
                enter =
                    fadeIn(tween(ANIMATION_DURATION)) +
                        expandHorizontally(tween(ANIMATION_DURATION)),
                exit =
                    fadeOut(tween(ANIMATION_DURATION)) +
                        shrinkHorizontally(tween(ANIMATION_DURATION)),
            ) {
                val defaultColors = ListItemDefaults.colors()
                Icon(
                    modifier = Modifier.padding(end = Dimens.smallPadding),
                    imageVector = Icons.Default.Check,
                    contentDescription = iconContentDescription,
                    // Set the tint explicitly here because the animation looks better if the icon
                    // does not change color to white while sliding out.
                    tint =
                        if (isEnabled) defaultColors.selectedHeadlineColor
                        else defaultColors.disabledHeadlineColor,
                )
            }
        },
        content = content,
        trailingContent = trailingContent,
    )
}

@Composable
fun SelectableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    isSelected: Boolean,
    isEnabled: Boolean = true,
    title: String,
    iconContentDescription: String? = null,
    onClick: (() -> Unit)? = null,
    testTag: String? = null,
    trailingContent: @Composable ((BoxScope.() -> Unit))? = null,
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isSelected = isSelected,
        isEnabled = isEnabled,
        iconContentDescription = iconContentDescription,
        testTag = testTag,
        onClick = onClick,
        content = { Text(title) },
        trailingContent = trailingContent,
    )
}

@Composable
fun SelectableListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    isSelected: Boolean,
    isEnabled: Boolean = true,
    title: String,
    subtitle: String,
    iconContentDescription: String? = null,
    onClick: (() -> Unit)? = null,
    testTag: String? = null,
    trailingContent: @Composable ((BoxScope.() -> Unit))? = null,
) {
    SelectableListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        isEnabled = isEnabled,
        iconContentDescription = iconContentDescription,
        isSelected = isSelected,
        testTag = testTag,
        onClick = onClick,
        content = {
            Column {
                Text(title)
                Text(
                    text = subtitle,
                    style = MaterialTheme.typography.labelLarge,
                    color =
                        if (isEnabled) MaterialTheme.colorScheme.onSurfaceVariant
                        else ListItemDefaults.colors().disabledHeadlineColor,
                )
            }
        },
        trailingContent = trailingContent,
    )
}

private const val ANIMATION_DURATION = 200
