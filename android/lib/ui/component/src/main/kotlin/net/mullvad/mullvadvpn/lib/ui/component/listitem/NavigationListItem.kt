package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
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
private fun PreviewNavigationListItem() {
    AppTheme {
        PreviewSpacedColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
            NavigationListItem(title = "Navigation sample", showWarning = false, onClick = {})
            NavigationListItem(
                hierarchy = Hierarchy.Child1,
                title = "Navigation sample",
                showWarning = true,
                onClick = {},
            )
            NavigationListItem(
                hierarchy = Hierarchy.Child1,
                title = "Navigation sample",
                subtitle = "Navigation sample",
                showWarning = false,
                onClick = {},
            )
        }
    }
}

@Suppress("ComposableLambdaParameterNaming")
@Composable
fun NavigationListItem(
    modifier: Modifier = Modifier,
    hierarchy: Hierarchy = Hierarchy.Parent,
    position: Position = Position.Single,
    title: String,
    subtitle: String? = null,
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
        content = { TitleAndSubtitle(title = title, subtitle = subtitle) },
        trailingContent = {
            Icon(
                Icons.AutoMirrored.Default.KeyboardArrowRight,
                contentDescription = stringResource(R.string.navigate),
            )
        },
    )
}
