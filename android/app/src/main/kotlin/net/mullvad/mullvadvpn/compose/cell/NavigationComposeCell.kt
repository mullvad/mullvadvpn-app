package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewNavigationCell() {
    AppTheme {
        NavigationComposeCell(
            title = "Navigation sample",
            bodyView = {
                NavigationCellBody(
                    contentBodyDescription = "",
                    content = "content body",
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                )
            },
            onClick = {},
            showWarning = true,
        )
    }
}

@Preview
@Composable
private fun PreviewExternalLinkComposeCell() {
    AppTheme {
        NavigationComposeCell(
            title = "External link sample",
            bodyView = {
                NavigationCellBody(
                    contentBodyDescription = "content body",
                    content = "content body",
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    isExternalLink = true,
                )
            },
            onClick = {},
            showWarning = false,
        )
    }
}

@Composable
fun NavigationComposeCell(
    title: String,
    modifier: Modifier = Modifier,
    showWarning: Boolean = false,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    bodyView: @Composable () -> Unit = {
        DefaultNavigationView(chevronContentDescription = title, tint = textColor)
    },
    isRowEnabled: Boolean = true,
    onClick: () -> Unit,
    testTag: String = "",
) {
    BaseCell(
        onCellClicked = onClick,
        headlineContent = {
            NavigationTitleView(
                title = title,
                modifier = modifier.weight(1f, true),
                showWarning = showWarning,
            )
        },
        bodyView = { bodyView() },
        isRowEnabled = isRowEnabled,
        testTag = testTag,
    )
}

@Composable
internal fun NavigationTitleView(
    title: String,
    modifier: Modifier = Modifier,
    showWarning: Boolean = false,
) {
    if (showWarning) {
        Icon(
            painter = painterResource(id = R.drawable.icon_alert),
            modifier = Modifier.padding(end = Dimens.smallPadding),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.error,
        )
    }
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
internal fun DefaultNavigationView(chevronContentDescription: String, tint: Color) {
    Icon(
        painter = painterResource(id = R.drawable.icon_chevron),
        contentDescription = chevronContentDescription,
        tint = tint,
    )
}

@Composable
internal fun DefaultExternalLinkView(chevronContentDescription: String, tint: Color) {
    Icon(
        painter = painterResource(id = R.drawable.icon_extlink),
        contentDescription = chevronContentDescription,
        tint = tint,
    )
}

@Composable
internal fun NavigationCellBody(
    content: String,
    contentBodyDescription: String,
    modifier: Modifier = Modifier,
    contentColor: Color = MaterialTheme.colorScheme.onPrimary,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    isExternalLink: Boolean = false,
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
    ) {
        Text(text = content, style = MaterialTheme.typography.labelMedium, color = textColor)
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        if (isExternalLink) {
            DefaultExternalLinkView(content, tint = contentColor)
        } else {
            DefaultNavigationView(
                chevronContentDescription = contentBodyDescription,
                tint = contentColor,
            )
        }
    }
}
