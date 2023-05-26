package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.dimensions.defaultDimensions
import net.mullvad.mullvadvpn.compose.theme.typeface.TypeScale

@Preview
@Composable
private fun PreviewNavigationCell() {
    NavigationComposeCell(
        title = "Navigation sample",
        bodyView = { NavigationCellBody("", "right body") },
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
    val textMediumSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    Text(
        text = title,
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = textMediumSize,
        color = Color.White,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
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
fun NavigationCellBody(title: String, content: String, modifier: Modifier = Modifier) {
    val textSize = TypeScale.TextSmall
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = Modifier.wrapContentWidth().wrapContentHeight()
    ) {
        Text(
            text = content,
            textAlign = TextAlign.Center,
            fontSize = textSize,
            color = Color.White,
            modifier =
                modifier
                    .padding(start = defaultDimensions.side_margin)
                    .wrapContentWidth(align = Alignment.End)
                    .wrapContentHeight()
        )
        Spacer(modifier = Modifier.width(defaultDimensions.side_margin))
        DefaultNavigationView(chevronContentDescription = title)
    }
}
