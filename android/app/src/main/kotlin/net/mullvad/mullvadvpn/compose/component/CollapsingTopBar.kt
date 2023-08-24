package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
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
import net.mullvad.mullvadvpn.lib.theme.AppTheme

@Preview
@Composable
private fun PreviewTopBar() {
    AppTheme {
        CollapsingTopBar(
            backgroundColor = MaterialTheme.colorScheme.secondary,
            onBackClicked = {},
            title = "View title",
            progress = 1.0f,
            modifier = Modifier.height(102.dp)
        )
    }
}

@Composable
fun CollapsingTopBar(
    backgroundColor: Color,
    onBackClicked: () -> Unit,
    title: String,
    progress: Float,
    modifier: Modifier,
    backIcon: Int? = null,
    shouldRotateBackButtonDown: Boolean = false
) {
    val expandedToolbarHeight = dimensionResource(id = R.dimen.expanded_toolbar_height)
    val iconSize = dimensionResource(id = R.dimen.icon_size)
    val sideMargin = dimensionResource(id = R.dimen.side_margin)
    val verticalMargin = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    val maxTopPadding = 48
    val minTopPadding = 14
    val maxTitleSize = 30
    val minTitleSize = 20

    Spacer(
        modifier = Modifier.fillMaxWidth().height(expandedToolbarHeight).background(backgroundColor)
    )

    Button(
        modifier = Modifier.wrapContentWidth().wrapContentHeight(),
        onClick = onBackClicked,
        colors =
            ButtonDefaults.buttonColors(
                contentColor = Color.White,
                containerColor = backgroundColor
            ),
        shape = MaterialTheme.shapes.small
    ) {
        Image(
            painter = painterResource(id = backIcon ?: R.drawable.icon_back),
            contentDescription = stringResource(id = R.string.back),
            modifier =
                Modifier.rotate(if (shouldRotateBackButtonDown) 270f else 0f)
                    .width(iconSize)
                    .height(iconSize)
        )
    }

    Text(
        text = title,
        style =
            TextStyle(color = Color.White, fontWeight = FontWeight.Bold, textAlign = TextAlign.End),
        modifier =
            modifier.padding(
                start = sideMargin,
                end = sideMargin,
                top = (minTopPadding + (maxTopPadding - minTopPadding) * progress).dp,
                bottom = verticalMargin
            ),
        fontSize =
            topBarSize(
                    progress = progress,
                    minTitleSize = minTitleSize,
                    maxTitleSize = maxTitleSize
                )
                .sp
    )
}

private fun topBarSize(progress: Float, minTitleSize: Int, maxTitleSize: Int): Float {
    return (minTitleSize + ((maxTitleSize - minTitleSize) * progress))
}
