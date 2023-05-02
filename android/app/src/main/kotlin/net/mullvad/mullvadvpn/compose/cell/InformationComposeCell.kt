package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.AlphaActive
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite

@Preview
@Composable
private fun PreviewInformationComposeCell() {
    InformationComposeCell(
        title = "Information row title",
        isEnabled = true,
        onCellClicked = {},
        onInfoClicked = {}
    )
}

@Composable
fun InformationComposeCell(
    title: String,
    isEnabled: Boolean = true,
    background: Color = MullvadBlue,
    onCellClicked: () -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val titleModifier = Modifier.alpha(if (isEnabled) AlphaActive else AlphaInactive)
    val bodyViewModifier = Modifier

    BaseCell(
        title = { SwitchCellTitle(title = title, modifier = titleModifier) },
        background = background,
        bodyView = {
            InformationComposeCellBody(modifier = bodyViewModifier, onInfoClicked = onInfoClicked)
        },
        onCellClicked = onCellClicked
    )
}

@Composable
private fun InformationComposeCellBody(modifier: Modifier, onInfoClicked: (() -> Unit)? = null) {
    val horizontalPadding = dimensionResource(id = R.dimen.medium_padding)
    val verticalPadding = 13.dp
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically
    ) {
        if (onInfoClicked != null) {
            Icon(
                modifier =
                    Modifier.clickable { onInfoClicked() }
                        .padding(
                            start = horizontalPadding,
                            end = horizontalPadding,
                            top = verticalPadding,
                            bottom = verticalPadding
                        )
                        .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = null,
                tint = MullvadWhite
            )
        }
    }
}
