package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewNavigationCell() {
    NavigationComposeCell(
        title = "Navigation sample",
        bodyView = {
            NavigationCellBody(
                chevronContentDescription = "",
                content = "right body",
                contentColor = Color.Red
            )
        },
        onClick = {}
    )
}

@Composable
fun NavigationComposeCell(
    title: String,
    bodyView: @Composable () -> Unit = { DefaultNavigationView(chevronContentDescription = title) },
    onClick: () -> Unit
) {
    BaseCell(
        onCellClicked = onClick,
        title = { NavigationTitleView(title = title) },
        bodyView = { bodyView() },
        subtitle = null,
    )
}

@Composable
private fun NavigationTitleView(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        color = MaterialTheme.colorScheme.onPrimary
    )
}

@Composable
private fun DefaultNavigationView(chevronContentDescription: String) {
    Image(
        painter = painterResource(id = R.drawable.icon_chevron),
        contentDescription = chevronContentDescription
    )
}

@Composable
fun NavigationCellBody(
    content: String,
    modifier: Modifier = Modifier,
    chevronContentDescription: String,
    contentColor: Color = MaterialTheme.colorScheme.onPrimary
) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.wrapContentWidth().wrapContentHeight()
    ) {
        Text(
            text = content.uppercase(),
            style = MaterialTheme.typography.labelMedium,
            color = contentColor
        )
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        DefaultNavigationView(chevronContentDescription = chevronContentDescription)
    }
}
