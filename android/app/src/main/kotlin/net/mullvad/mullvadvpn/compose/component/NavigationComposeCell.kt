package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun SplitTunnelingNavigationComposeCellPreview() {
    NavigationComposeCell(
        title = stringResource(id = R.string.split_tunneling),
        onClick = {}
    )
}

@Composable
fun NavigationComposeCell(
    title: String,
    modifier: Modifier = Modifier,
    bodyView: @Composable () -> Unit = { DefaultNavigationView() },
    onClick: () -> Unit
) {
    BaseCell(
        onCellClicked = onClick,
        title = { NavigationTitleView(title = title, modifier = modifier) },
        bodyView = {
            bodyView()
        },
        subtitle = null,
    )
}

@Composable
private fun NavigationTitleView(
    title: String,
    modifier: Modifier = Modifier
) {
    Text(
        text = title,
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = 18.sp,
        color = Color.White,
        modifier = modifier
            .wrapContentWidth(align = Alignment.End)
            .wrapContentHeight()
    )
}

@Composable
private fun DefaultNavigationView() {
    Image(painter = painterResource(id = R.drawable.icon_chevron), contentDescription = "")
}
