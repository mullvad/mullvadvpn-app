package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R

@Composable
fun TopBar(
    backgroundColor: Color,
    onSettingsClicked: () -> Unit,
    modifier: Modifier = Modifier
) {
    ConstraintLayout(
        modifier = Modifier
            .fillMaxWidth()
            .height(dimensionResource(id = R.dimen.top_bar_height))
            .background(backgroundColor)
            .then(modifier),
    ) {
        val (logo, appName, settingsIcon) = createRefs()

        Image(
            painter = painterResource(id = R.drawable.logo_icon),
            contentDescription = null, // No meaningful user info or action.
            modifier = Modifier
                .width(44.dp)
                .height(44.dp)
                .constrainAs(logo) {
                    start.linkTo(parent.start, margin = 16.dp)
                    top.linkTo(parent.top, margin = 12.dp)
                    bottom.linkTo(parent.bottom, margin = 12.dp)
                }
        )

        CapsText(
            text = stringResource(id = R.string.app_name),
            fontWeight = FontWeight.Bold,
            fontSize = 24.sp,
            color = colorResource(id = R.color.white80),
            modifier = Modifier
                .constrainAs(appName) {
                    start.linkTo(logo.end, margin = 9.dp)
                    top.linkTo(parent.top, margin = 12.dp)
                }
        )

        Icon(
            painter = painterResource(R.drawable.icon_settings),
            contentDescription = stringResource(id = R.string.settings),
            tint = Color.White,

            modifier = Modifier
                .clickable { onSettingsClicked() }
                .fillMaxHeight()
                .padding(horizontal = 16.dp)
                .constrainAs(settingsIcon) {
                    end.linkTo(parent.end)
                }
        )
    }
}
