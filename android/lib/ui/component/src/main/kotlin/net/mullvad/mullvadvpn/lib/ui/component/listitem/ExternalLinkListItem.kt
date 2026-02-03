package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewExternalLinkListItem() {
    AppTheme {
        PreviewSpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            ExternalLinkListItem(title = "Navigation sample", showWarning = false, onClick = {})
            ExternalLinkListItem(
                hierarchy = Hierarchy.Child1,
                title = "Navigation sample",
                showWarning = true,
                onClick = {},
            )
        }
    }
}

@Composable
fun ExternalLinkListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    subtitle: String? = null,
    subTitleTextDirection: TextDirection = TextDirection.Unspecified,
    leadingIcon: ImageVector? = null,
    colors: ListItemColors = ListItemDefaults.colors(),
    showWarning: Boolean = false,
    isRowEnabled: Boolean = true,
    onClick: () -> Unit,
    testTag: String? = null,
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        onClick = onClick,
        colors = colors,
        isEnabled = isRowEnabled,
        testTag = testTag,
        leadingContent = {
            when {
                showWarning ->
                    Icon(
                        imageVector = Icons.Default.Error,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        contentDescription = stringResource(R.string.warning),
                        tint = MaterialTheme.colorScheme.error,
                    )
                leadingIcon != null ->
                    Icon(
                        imageVector = leadingIcon,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        contentDescription = null,
                    )
            }
        },
        content = {
            TitleAndSubtitle(
                title = title,
                subtitle = subtitle,
                subTitleTextDirection = subTitleTextDirection,
            )
        },
        trailingContent = {
            Icon(
                imageVector = Icons.AutoMirrored.Default.OpenInNew,
                contentDescription = stringResource(R.string.external_link),
            )
        },
    )
}
