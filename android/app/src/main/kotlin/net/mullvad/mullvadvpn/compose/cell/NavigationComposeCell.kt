package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
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
                    contentColor = MaterialTheme.colorScheme.error,
                )
            },
            onClick = {},
            showWarning = true
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
                    contentColor = MaterialTheme.colorScheme.onSecondary,
                    isExternalLink = true
                )
            },
            onClick = {},
            showWarning = false
        )
    }
}

@Composable
fun NavigationComposeCell(
    title: String,
    modifier: Modifier = Modifier,
    showWarning: Boolean = false,
    bodyView: @Composable () -> Unit = { DefaultNavigationView(chevronContentDescription = title) },
    isRowEnabled: Boolean = true,
    onClick: () -> Unit
) {
    BaseCell(
        onCellClicked = onClick,
        headlineContent = {
            NavigationTitleView(
                title = title,
                modifier = modifier.weight(1f, true),
                showWarning = showWarning
            )
        },
        bodyView = { bodyView() },
        isRowEnabled = isRowEnabled
    )
}

@Composable
internal fun NavigationTitleView(
    title: String,
    modifier: Modifier = Modifier,
    showWarning: Boolean = false
) {
    if (showWarning) {
        Image(
            painter = painterResource(id = R.drawable.icon_alert),
            modifier = Modifier.padding(end = Dimens.smallPadding),
            contentDescription = stringResource(id = R.string.update_available)
        )
    }
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier
    )
}

@Composable
internal fun DefaultNavigationView(chevronContentDescription: String) {
    Image(
        painter = painterResource(id = R.drawable.icon_chevron),
        contentDescription = chevronContentDescription
    )
}

@Composable
internal fun DefaultExternalLinkView(chevronContentDescription: String) {
    Image(
        painter = painterResource(id = R.drawable.icon_extlink),
        contentDescription = chevronContentDescription
    )
}

@Composable
internal fun NavigationCellBody(
    content: String,
    contentBodyDescription: String,
    modifier: Modifier = Modifier,
    contentColor: Color = MaterialTheme.colorScheme.onSecondary,
    isExternalLink: Boolean = false
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.wrapContentWidth().wrapContentHeight()
    ) {
        Text(text = content, style = MaterialTheme.typography.labelMedium, color = contentColor)
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        if (isExternalLink) {
            DefaultExternalLinkView(content)
        } else {
            DefaultNavigationView(chevronContentDescription = contentBodyDescription)
        }
    }
}
