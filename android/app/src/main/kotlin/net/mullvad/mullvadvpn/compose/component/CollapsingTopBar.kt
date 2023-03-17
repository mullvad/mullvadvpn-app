package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewTopBar() {
    CollapsingTopBar(
        backgroundColor = MullvadDarkBlue,
        onBackClicked = {},
        title = "View title",
        progress = 1.0f,
        backTitle = "Back",
        modifier = Modifier.height(102.dp)
    )
}

@Composable
fun CollapsingTopBar(
    backgroundColor: Color,
    onBackClicked: () -> Unit,
    title: String,
    progress: Float,
    backTitle: String,
    modifier: Modifier
) {
    val expandedToolbarHeight = dimensionResource(id = R.dimen.expanded_toolbar_height)
    val iconSize = dimensionResource(id = R.dimen.icon_size)
    val iconPadding = dimensionResource(id = R.dimen.small_padding)
    val sideMargin = dimensionResource(id = R.dimen.side_margin)
    val verticalMargin = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    val textSize = dimensionResource(id = R.dimen.text_small).value.sp
    val maxTopPadding = 48
    val minTopPadding = 14
    val maxTitleSize = 30
    val minTitleSize = 20

    Spacer(
        modifier = Modifier
            .fillMaxWidth()
            .height(expandedToolbarHeight)
            .background(backgroundColor)
    )

    Button(
        modifier = Modifier
            .wrapContentWidth()
            .wrapContentHeight(),
        onClick = onBackClicked,
        colors = ButtonDefaults.buttonColors(
            contentColor = Color.White,
            backgroundColor = MullvadDarkBlue
        )
    ) {
        Image(
            painter = painterResource(id = R.drawable.icon_back),
            contentDescription = stringResource(id = R.string.back),
            modifier = Modifier
                .width(iconSize)
                .height(iconSize)
        )
        Spacer(
            modifier = Modifier
                .width(iconPadding)
                .fillMaxHeight()
        )
        Text(
            text = backTitle,
            color = MullvadWhite60,
            fontWeight = FontWeight.Bold,
            fontSize = textSize
        )
    }

    Text(
        text = title,
        style = TextStyle(
            color = Color.White,
            fontWeight = FontWeight.Bold,
            textAlign = TextAlign.End
        ),
        modifier = modifier
            .padding(
                start = sideMargin,
                end = sideMargin,
                top = (minTopPadding + (maxTopPadding - minTopPadding) * progress).dp,
                bottom = verticalMargin
            ),
        fontSize = topBarSize(
            progress = progress,
            minTitleSize = minTitleSize,
            maxTitleSize = maxTitleSize
        ).sp
    )
}

private fun topBarSize(progress: Float, minTitleSize: Int, maxTitleSize: Int): Float {
    return (minTitleSize + ((maxTitleSize - minTitleSize) * progress))
}
