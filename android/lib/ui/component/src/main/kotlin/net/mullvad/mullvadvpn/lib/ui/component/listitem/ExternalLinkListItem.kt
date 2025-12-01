package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxScope
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewSpacedColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position

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
    showWarning: Boolean = false,
    isRowEnabled: Boolean = true,
    onClick: () -> Unit,
    testTag: String? = null,
    icon: @Composable ((BoxScope) -> Unit) = {
        Icon(imageVector = Icons.AutoMirrored.Default.OpenInNew, contentDescription = title)
    },
) {
    ExternalLinkListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        title = @Composable { Text(title) },
        showWarning = showWarning,
        isRowEnabled = isRowEnabled,
        onClick = onClick,
        testTag = testTag,
        icon = icon,
    )
}

@Composable
fun ExternalLinkListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: @Composable (BoxScope) -> Unit,
    showWarning: Boolean = false,
    isRowEnabled: Boolean = true,
    onClick: () -> Unit,
    testTag: String? = null,
    contentDescription: String? = null,
    icon: @Composable ((BoxScope) -> Unit) = {
        Icon(
            imageVector = Icons.AutoMirrored.Default.OpenInNew,
            contentDescription = contentDescription,
        )
    },
) {
    MullvadListItem(
        modifier = modifier,
        hierarchy = hierarchy,
        position = position,
        onClick = onClick,
        isEnabled = isRowEnabled,
        testTag = testTag,
        leadingContent = {
            if (showWarning) {
                Icon(
                    imageVector = Icons.Default.Error,
                    modifier = Modifier.padding(end = Dimens.smallPadding),
                    contentDescription = stringResource(R.string.warning),
                    tint = MaterialTheme.colorScheme.error,
                )
            }
        },
        content = title,
        trailingContent = icon,
    )
}
