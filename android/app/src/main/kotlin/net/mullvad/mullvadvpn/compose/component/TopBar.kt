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
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R

@Preview
@Composable
fun PreviewTopBar() {
    TopBar(
        backgroundColor = colorResource(R.color.blue),
        onSettingsClicked = {}
    )
}

@Composable
fun TopBar(
    backgroundColor: Color,
    onSettingsClicked: () -> Unit,
    modifier: Modifier = Modifier,
    isIconAndLogoVisible: Boolean = true
) {
    ConstraintLayout(
        modifier = Modifier
            .fillMaxWidth()
            .height(dimensionResource(id = R.dimen.top_bar_height))
            .background(backgroundColor)
            .then(modifier),
    ) {
        val (logo, appName, settingsIcon) = createRefs()

        if (isIconAndLogoVisible) {
            Image(
                painter = painterResource(id = R.drawable.logo_icon),
                contentDescription = null, // No meaningful user info or action.
                modifier = Modifier
                    .width(44.dp)
                    .height(44.dp)
                    .constrainAs(logo) {
                        centerVerticallyTo(parent)
                        start.linkTo(parent.start, margin = 16.dp)
                    }
            )

            Icon(
                painter = painterResource(id = R.drawable.logo_text),
                tint = colorResource(id = R.color.white80),
                contentDescription = null, // No meaningful user info or action.
                modifier = Modifier
                    .height(16.dp)
                    .constrainAs(appName) {
                        centerVerticallyTo(parent)
                        start.linkTo(logo.end, margin = 8.dp)
                    }
            )
        }

        Image(
            painter = painterResource(R.drawable.icon_settings),
            contentDescription = stringResource(id = R.string.settings),
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
