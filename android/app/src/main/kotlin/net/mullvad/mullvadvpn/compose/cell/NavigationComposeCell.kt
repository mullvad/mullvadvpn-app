package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewNavigationCell() {
    AppTheme {
        NavigationComposeCell(
            title = "Navigation sample",
            showWarning = true,
            onClick = {},
            bodyView = {
                NavigationCellBody(
                    contentBodyDescription = "",
                    content = "content body",
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                )
            },
        )
    }
}

@Preview
@Composable
private fun PreviewExternalLinkComposeCell() {
    AppTheme {
        NavigationComposeCell(
            title = "External link sample",
            onClick = {},
            bodyView = {
                NavigationCellBody(
                    contentBodyDescription = "content body",
                    content = "content body",
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    isExternalLink = true,
                )
            },
        )
    }
}

@Suppress("ComposableLambdaParameterNaming")
@Composable
fun NavigationComposeCell(
    title: String,
    modifier: Modifier = Modifier,
    showWarning: Boolean = false,
    textColor: Color = MaterialTheme.colorScheme.onPrimary,
    textStyle: TextStyle = MaterialTheme.typography.titleMedium,
    isRowEnabled: Boolean = true,
    onClick: () -> Unit,
    testTag: String = "",
    bodyView: @Composable () -> Unit = {
        Icon(
            Icons.AutoMirrored.Default.KeyboardArrowRight,
            contentDescription = title,
            tint = textColor,
        )
    },
) {
    BaseCell(
        modifier = modifier,
        onCellClicked = onClick,
        headlineContent = {
            NavigationTitleView(
                title = title,
                style = textStyle,
                modifier = Modifier.weight(1f, true),
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
    style: TextStyle = MaterialTheme.typography.titleMedium,
) {
    if (showWarning) {
        Icon(
            imageVector = Icons.Default.Error,
            modifier = Modifier.padding(end = Dimens.smallPadding),
            contentDescription = null,
            tint = MaterialTheme.colorScheme.error,
        )
    }
    Text(
        text = title,
        style = style,
        color = MaterialTheme.colorScheme.onPrimary,
        modifier = modifier,
        maxLines = 1,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
internal fun DefaultExternalLinkView(chevronContentDescription: String, tint: Color) {
    Icon(
        imageVector = Icons.AutoMirrored.Default.OpenInNew,
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
        Text(text = content, style = MaterialTheme.typography.titleMedium, color = textColor)
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        if (isExternalLink) {
            DefaultExternalLinkView(content, tint = contentColor)
        } else {
            Icon(
                Icons.AutoMirrored.Default.KeyboardArrowRight,
                tint = contentColor,
                contentDescription = contentBodyDescription,
            )
        }
    }
}
