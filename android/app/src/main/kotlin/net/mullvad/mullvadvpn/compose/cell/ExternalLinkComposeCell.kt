package net.mullvad.mullvadvpn.compose.cell

import android.net.Uri
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.ui.extension.openLink

@Preview
@Composable
private fun PreviewNExternalLinkComposeCell() {
    ExternalLinkComposeCell(
        title = "Navigation sample",
        uri = Uri.parse("www.mullvad.net"),
        showWarning = true
    )
}

@Composable
fun ExternalLinkComposeCell(
    title: String,
    uri: Uri,
    showWarning: Boolean = false,
    bodyView: @Composable () -> Unit = {
        DefaultExternalLinkView(chevronContentDescription = title)
    }
) {
    val context = LocalContext.current
    BaseCell(
        onCellClicked = { context.openLink(uri) },
        title = { ExternalLinkTitleView(title = title, showWarning = showWarning) },
        bodyView = { bodyView() },
        subtitle = null
    )
}

@Composable
private fun ExternalLinkTitleView(title: String, showWarning: Boolean = false) {
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
        color = MaterialTheme.colorScheme.onPrimary
    )
}

@Composable
private fun DefaultExternalLinkView(chevronContentDescription: String) {
    Image(
        painter = painterResource(id = R.drawable.icon_extlink),
        contentDescription = chevronContentDescription
    )
}

@Composable
fun ExternalLinkCellBody(title: String, content: String) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.wrapContentWidth().wrapContentHeight()
    ) {
        Text(
            text = content,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondary
        )
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        DefaultExternalLinkView(title)
    }
}
