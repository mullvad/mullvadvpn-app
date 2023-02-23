package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar

@Preview
@Composable
private fun PreviewLoadingScreen() {
    LoadingScreen {}
}

@Composable
fun LoadingScreen(
    onSettingsCogClicked: () -> Unit
) {
    val backgroundColor = colorResource(id = R.color.blue)

    ScaffoldWithTopBar(
        topBarColor = backgroundColor,
        statusBarColor = backgroundColor,
        navigationBarColor = backgroundColor,
        onSettingsClicked = onSettingsCogClicked,
        isIconAndLogoVisible = false,
        content = {
            Box(
                contentAlignment = Alignment.Center,
                modifier = Modifier
                    .background(backgroundColor)
                    .padding(bottom = 64.dp)
                    .fillMaxSize()
            ) {
                Column(
                    verticalArrangement = Arrangement.Center,
                    horizontalAlignment = Alignment.CenterHorizontally
                ) {
                    Image(
                        painter = painterResource(id = R.drawable.launch_logo),
                        contentDescription = "",
                        modifier = Modifier
                            .size(120.dp)
                    )
                    Image(
                        painter = painterResource(id = R.drawable.logo_text),
                        contentDescription = "",
                        alpha = 0.6f,
                        modifier = Modifier
                            .padding(top = 12.dp)
                            .height(18.dp)
                    )
                    Text(
                        text = stringResource(id = R.string.connecting_to_daemon),
                        fontSize = 13.sp,
                        color = colorResource(id = R.color.white40),
                        modifier = Modifier
                            .padding(top = 12.dp)
                    )
                }
            }
        }
    )
}
