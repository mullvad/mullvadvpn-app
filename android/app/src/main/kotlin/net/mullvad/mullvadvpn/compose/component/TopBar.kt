package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewTopBar() {
    AppTheme {
        TopBar(
            backgroundColor = MaterialTheme.colorScheme.inversePrimary,
            iconTintColor = MaterialTheme.colorScheme.onPrimary,
            onSettingsClicked = null,
            onAccountClicked = {}
        )
    }
}

@Composable
fun TopBar(
    backgroundColor: Color,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    modifier: Modifier = Modifier,
    iconTintColor: Color,
    isIconAndLogoVisible: Boolean = true
) {
    Row(
        modifier =
            Modifier.fillMaxWidth()
                .height(Dimens.topBarHeight)
                .background(backgroundColor)
                .then(modifier),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (isIconAndLogoVisible) {
            Row(
                Modifier.height(Dimens.topBarHeight)
                    .weight(1f)
                    .padding(start = Dimens.mediumPadding),
                verticalAlignment = Alignment.CenterVertically
            ) {
                Image(
                    painter = painterResource(id = R.drawable.logo_icon),
                    contentDescription = null, // No meaningful user info or action.
                    modifier = Modifier.size(Dimens.buttonHeight)
                )
                Icon(
                    painter = painterResource(id = R.drawable.logo_text),
                    tint = iconTintColor,
                    contentDescription = null, // No meaningful user info or action.
                    modifier =
                        Modifier.padding(start = Dimens.smallPadding).height(Dimens.mediumPadding)
                )
            }
        }

        if (onAccountClicked != null) {
            Image(
                painter = painterResource(R.drawable.icon_account),
                contentDescription = stringResource(id = R.string.settings_account),
                modifier =
                    Modifier.clickable { onAccountClicked() }
                        .fillMaxHeight()
                        .padding(horizontal = Dimens.mediumPadding)
            )
        }

        if (onSettingsClicked != null) {
            Image(
                painter = painterResource(R.drawable.icon_settings),
                contentDescription = stringResource(id = R.string.settings),
                modifier =
                    Modifier.clickable { onSettingsClicked() }
                        .fillMaxHeight()
                        .padding(horizontal = Dimens.mediumPadding)
            )
        }
    }
}
