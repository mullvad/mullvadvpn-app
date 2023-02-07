package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material.Button
import androidx.compose.material.ButtonDefaults
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
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

@Preview
@Composable
fun previewTopBar(){
    CollapsingTopBar(
        backgroundColor = MullvadDarkBlue,
        onBackClicked = {},
        title = stringResource(id = R.string.settings_advanced),
        progress = 1.0f,
        backTitle = stringResource(id = R.string.settings),
        scaffoldModifier = Modifier
    )

}

@Composable
fun CollapsingTopBar(
    backgroundColor: Color,
    onBackClicked: () -> Unit,
    title: String,
    progress: Float,
    backTitle: String,
    scaffoldModifier: Modifier
) {

    Spacer(
        modifier = Modifier
            .fillMaxWidth()
            .height(104.dp)
            .background(backgroundColor)
    )

    Button(
        modifier = Modifier
            .widthIn(min = 32.dp)
            .height(50.dp),
        onClick = onBackClicked,
        colors = ButtonDefaults.buttonColors(
            contentColor = Color.White,
            backgroundColor = colorResource(id = R.color.darkBlue)
        )
    ) {
        Image(
            painter = painterResource(id = R.drawable.icon_back),
            contentDescription = "",
            modifier = Modifier
                .width(24.dp)
                .height(24.dp)
        )
        Spacer(
            modifier = Modifier
                .width(8.dp)
                .fillMaxHeight()
        )
        Text(
            text = backTitle,
            color = colorResource(id = R.color.white60),
            fontWeight = FontWeight.Bold,
            fontSize = 13.sp
        )
    }

    Text(
        text = title,
        style = TextStyle(
            color = Color.White,
            fontWeight = FontWeight.Bold,
            textAlign = TextAlign.End
        ),
        modifier = scaffoldModifier
            .padding(start = 22.dp, end = 22.dp, top = (12 + (48 - 12) * progress).dp, bottom = 12.dp),
        fontSize = (20 + (30 - 20) * progress).sp
    )
}
