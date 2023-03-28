package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CellSwitch
import net.mullvad.mullvadvpn.compose.component.HtmlText
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.theme.AlphaActive
import net.mullvad.mullvadvpn.compose.theme.AlphaInactive
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewSwitchComposeCell() {
    SwitchComposeCell(
        title = "Checkbox Title",
        isEnabled = true,
        isToggled = true,
        onCellClicked = {},
        onInfoClicked = {}
    )
}

@Composable
fun SwitchComposeCell(
    title: String,
    isToggled: Boolean,
    isEnabled: Boolean = true,
    background: Color = MullvadBlue,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val alpha = if (isEnabled) AlphaActive else AlphaInactive
    val titleModifier = Modifier.alpha(alpha)
    val bodyViewModifier = Modifier.alpha(alpha)
    val subtitleModifier = Modifier

    BaseCell(
        title = { SwitchCellTitle(title = title, modifier = titleModifier) },
        isRowEnabled = isEnabled,
        bodyView = {
            SwitchCellView(
                onSwitchClicked = null,
                isEnabled = isEnabled,
                isToggled = isToggled,
                modifier = bodyViewModifier,
                onInfoClicked = onInfoClicked
            )
        },
        background = background,
        onCellClicked = { onCellClicked(!isToggled) },
        subtitleModifier = subtitleModifier
    )
}

@Composable
fun SwitchCellTitle(title: String, modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    Text(
        text = title,
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = textSize,
        color = MullvadWhite,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
    )
}

@Composable
fun SwitchCellView(
    isEnabled: Boolean,
    isToggled: Boolean,
    modifier: Modifier,
    onSwitchClicked: ((Boolean) -> Unit)? = null,
    onInfoClicked: (() -> Unit)? = null
) {
    val horizontalPadding = dimensionResource(id = R.dimen.medium_padding)
    val verticalPadding = 13.dp
    Row(
        modifier = modifier.wrapContentWidth().wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onInfoClicked != null) {
            Icon(
                modifier =
                    Modifier.clickable { onInfoClicked() }
                        .padding(
                            start = horizontalPadding,
                            end = horizontalPadding,
                            top = verticalPadding,
                            bottom = verticalPadding,
                        )
                        .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = stringResource(id = R.string.confirm_local_dns),
                tint = MullvadWhite
            )
        }

        CellSwitch(isChecked = isToggled, isEnabled = isEnabled, onCheckedChange = onSwitchClicked)
    }
}

@Composable
fun CustomDnsCellSubtitle(isCellClickable: Boolean, modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_small).value

    HtmlText(
        htmlFormattedString =
            textResource(
                if (isCellClickable) {
                    R.string.custom_dns_footer
                } else {
                    R.string.custom_dns_disable_mode_subtitle
                }
            ),
        textSize = textSize,
        textColor = MullvadWhite60.toArgb(),
        modifier = modifier
    )
}
