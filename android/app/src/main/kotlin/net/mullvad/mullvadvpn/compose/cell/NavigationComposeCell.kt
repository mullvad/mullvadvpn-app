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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
private fun PreviewNavigationCell() {
    NavigationComposeCell(
        title = "Navigation sample",
        bodyView = { NavigationCellBody("", "right body", Modifier, true) },
        onClick = {}
    )
}

@Composable
fun NavigationComposeCell(
    title: String,
    modifier: Modifier = Modifier,
    bodyView: @Composable () -> Unit = { DefaultNavigationView(chevronContentDescription = title) },
    onClick: () -> Unit
) {
    BaseCell(
        onCellClicked = onClick,
        title = { NavigationTitleView(title = title, modifier = modifier) },
        bodyView = { bodyView() },
        subtitle = null,
    )
}

@Composable
private fun NavigationTitleView(title: String, modifier: Modifier = Modifier) {
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
    title: String,
    content: String,
    modifier: Modifier = Modifier,
    isTypeOfContentError: Boolean = false
) {

    val colors = listOf(MaterialTheme.colorScheme.onPrimary, MaterialTheme.colorScheme.error)
    var colorsText by remember { mutableStateOf(colors[0]) }

    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.wrapContentWidth().wrapContentHeight()
    ) {
        colorsText = colors[if (isTypeOfContentError) 1 else 0]
        Text(
            text = content.uppercase(),
            style = MaterialTheme.typography.labelMedium,
            color = colorsText
        )
        Spacer(modifier = Modifier.width(Dimens.sideMargin))
        DefaultNavigationView(chevronContentDescription = title)
    }
}
