package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CellSwitch
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

@Preview
@Composable
private fun PreviewDnsComposeCell() {
    CustomDnsComposeCell(checkboxDefaultState = true, onToggle = {})
}

@Composable
fun CustomDnsComposeCell(checkboxDefaultState: Boolean, onToggle: (Boolean) -> Unit) {
    val titleModifier = Modifier
    val bodyViewModifier = Modifier
    val subtitleModifier = Modifier

    BaseCell(
        title = { CustomDnsCellTitle(modifier = titleModifier) },
        bodyView = {
            CustomDnsCellView(
                switchTriggered = { onToggle(it) },
                isToggled = checkboxDefaultState,
                modifier = bodyViewModifier
            )
        },
        onCellClicked = { onToggle(!checkboxDefaultState) },
        subtitleModifier = subtitleModifier
    )
}

@Composable
fun CustomDnsCellTitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    Text(
        text = stringResource(R.string.enable_custom_dns),
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = textSize,
        color = MullvadWhite,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
    )
}

@Composable
fun CustomDnsCellView(switchTriggered: (Boolean) -> Unit, isToggled: Boolean, modifier: Modifier) {
    Row(modifier = modifier.wrapContentWidth().wrapContentHeight()) {
        CellSwitch(isChecked = isToggled, onCheckedChange = null)
    }
}

@Composable
fun CustomDnsCellSubtitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_small).value.sp
    Text(
        text = stringResource(R.string.custom_dns_footer),
        fontSize = textSize,
        color = MullvadWhite60,
        modifier = modifier
    )
}
