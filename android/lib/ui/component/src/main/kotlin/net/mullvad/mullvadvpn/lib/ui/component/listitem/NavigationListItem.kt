package net.mullvad.mullvadvpn.lib.ui.component.listitem

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.KeyboardArrowRight
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.ui.component.R
import net.mullvad.mullvadvpn.lib.ui.component.preview.PreviewColumn
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.warning
import net.mullvad.mullvadvpn.lib.ui.util.applyIf

@Preview
@Composable
private fun PreviewNavigationListItem() {
    PreviewColumn(Modifier.background(MaterialTheme.colorScheme.surface)) {
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
        NavigationListItem(
            hierarchy = Hierarchy.Child1,
            title = "Navigation sample",
            subtitle = "Navigation sample",
            showNotificationDot = true,
            onClick = {},
        )
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
    singeLine: Boolean = true,
    subTitleTextDirection: TextDirection = TextDirection.Unspecified,
    showWarning: Boolean = false,
    showNotificationDot: Boolean = false,
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
                    imageVector = Icons.Rounded.Error,
                    modifier = Modifier.padding(end = Dimens.smallPadding),
                    contentDescription = stringResource(R.string.warning),
                    tint = MaterialTheme.colorScheme.error,
                )
            }
        },
        content = {
            TitleAndSubtitle(
                title = title,
                subtitle = subtitle,
                subTitleTextDirection = subTitleTextDirection,
                singleLine = singeLine,
            )
        },
        trailingContent = {
            Row(
                modifier =
                    Modifier.applyIf(showNotificationDot) {
                        padding(end = Dimens.notificationDotSize)
                    }
            ) {
                if (showNotificationDot) {
                    Box(
                        modifier =
                            Modifier.align(Alignment.CenterVertically)
                                .size(Dimens.notificationDotSize)
                                .background(MaterialTheme.colorScheme.warning, CircleShape)
                    )
                }
                Icon(
                    modifier = Modifier,
                    imageVector = Icons.AutoMirrored.Rounded.KeyboardArrowRight,
                    contentDescription = stringResource(R.string.navigate),
                )
            }
        },
    )
}
