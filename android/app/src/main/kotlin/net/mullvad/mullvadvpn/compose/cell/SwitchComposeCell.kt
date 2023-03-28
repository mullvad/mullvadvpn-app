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
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewSwitchComposeCell() {
    SwitchComposeCell(
        title = "Checkbox Title",
        checkboxEnableState = true,
        checkboxDefaultState = true,
        onCellClicked = {},
        onInfoClicked = {},
    )
}

@Composable
fun SwitchComposeCell(
    title: String,
    checkboxDefaultState: Boolean,
    checkboxEnableState: Boolean = true,
    background: Color = MullvadBlue,
    onCellClicked: (Boolean) -> Unit = {},
    onInfoClicked: (() -> Unit)? = null
) {
    val alpha = if (checkboxEnableState) 1f else 0.3f
    val titleModifier = Modifier.alpha(alpha)
    val bodyViewModifier = Modifier.alpha(alpha)
    val subtitleModifier = Modifier

    BaseCell(
        title = { SwitchCellTitle(title = title, modifier = titleModifier) },
        isRowEnabled = checkboxEnableState,
        bodyView = {
            SwitchCellView(
                switchTriggered = null,
                isEnabled = checkboxEnableState,
                isToggled = checkboxDefaultState,
                modifier = bodyViewModifier,
                onInfoClicked = onInfoClicked,
            )
        },
        background = background,
        onCellClicked = { onCellClicked(!checkboxDefaultState) },
        subtitleModifier = subtitleModifier,
    )
}

@Composable
fun SwitchCellTitle(
    title: String,
    modifier: Modifier
) {
    val textSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    Text(
        text = title,
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = textSize,
        color = MullvadWhite,
        modifier = modifier
            .wrapContentWidth(align = Alignment.End)
            .wrapContentHeight(),
    )
}

@Composable
fun SwitchCellView(
    isEnabled: Boolean,
    isToggled: Boolean,
    modifier: Modifier,
    switchTriggered: ((Boolean) -> Unit)? = null,
    onInfoClicked: (() -> Unit)? = null
) {
    val horizontalPadding = dimensionResource(id = R.dimen.medium_padding)
    val verticalPadding = 13.dp
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight(),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (onInfoClicked != null) {

            Icon(
                modifier = Modifier
                    .clickable { onInfoClicked() }
                    .padding(
                        start = horizontalPadding,
                        end = horizontalPadding,
                        top = verticalPadding,
                        bottom = verticalPadding,
                    )
                    .align(Alignment.CenterVertically),
                painter = painterResource(id = R.drawable.icon_info),
                contentDescription = stringResource(id = R.string.confirm_local_dns),
                tint = MullvadWhite,
            )
        }

        CellSwitch(
            isChecked = isToggled,
            isEnabled = isEnabled,
            onCheckedChange = switchTriggered,
        )
    }
}

@Composable
fun CustomDnsCellSubtitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_small).value.sp
    Text(
        text = stringResource(R.string.custom_dns_footer),
        fontSize = textSize,
        color = MullvadWhite60,
        modifier = modifier,
    )
}
