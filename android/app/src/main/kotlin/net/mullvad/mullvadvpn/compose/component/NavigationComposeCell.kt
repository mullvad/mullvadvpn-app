package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun previewSplitTunneling() {
    NavigationComposeCell(
        title = stringResource(id = R.string.split_tunneling),
        onClick = {}
    )
}

@Composable
fun NavigationComposeCell(
    title: String,
    bodyView: @Composable () -> Unit = { defaultNavigationView() },
    bodyViewModifier: Modifier = Modifier,
    onClick: () -> Unit
) {
    BaseCell(
        uiState = null,
        title = { navigationTitleView(title = title) },
        bodyView = {
            bodyView()
        },
        subtitle = null,
        onCellClicked = onClick
    )
}

@Composable
fun navigationTitleView(
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
fun defaultNavigationView() {
    Image(painter = painterResource(id = R.drawable.icon_chevron), contentDescription = "")
}

@Composable
fun navigationBodyView(
    defaultValue: Int?,
    onMtuChanged: (String) -> Unit,
    modifier: Modifier
) {
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight()
    ) {
        TextField(
            value = defaultValue.toString(),
            onValueChange = onMtuChanged,
            placeholder = {
                Text(
                    text = stringResource(id = R.string.hint_default),
                    color = colorResource(
                        id = R.color.white60
                    )
                )
            },
            modifier = Modifier
                .width(96.dp)
                .height(32.dp)
                .background(colorResource(id = R.color.white10), shape = RoundedCornerShape(4.dp)),
            colors = TextFieldDefaults.textFieldColors(
                backgroundColor = colorResource(id = R.color.white10),
                focusedIndicatorColor = Color.Transparent, // hide the indicator
                unfocusedIndicatorColor = colorResource(id = R.color.white20),
                textColor = colorResource(id = R.color.white),

            )

        )
    }
}
