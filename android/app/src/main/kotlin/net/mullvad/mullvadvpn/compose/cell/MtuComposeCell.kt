package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth as wrapContentWidth1
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60
import net.mullvad.mullvadvpn.constant.MTU_MAX_VALUE
import net.mullvad.mullvadvpn.constant.MTU_MIN_VALUE

@Preview
@Composable
fun MtuComposeCellPreview() {
    MtuComposeCell(mtuValue = "1300", onEditMtu = {})
}

@Composable
fun MtuComposeCell(
    mtuValue: String,
    onEditMtu: () -> Unit,
) {
    val titleModifier = Modifier
    val subtitleModifier = Modifier

    BaseCell(
        title = { MtuTitle(modifier = titleModifier) },
        bodyView = { MtuBodyView(mtuValue = mtuValue, modifier = titleModifier) },
        subtitle = { MtuSubtitle(subtitleModifier) },
        subtitleModifier = subtitleModifier,
        onCellClicked = { onEditMtu.invoke() }
    )
}

@Composable
private fun MtuTitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_medium_plus).value.sp
    Text(
        text = stringResource(R.string.wireguard_mtu),
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = textSize,
        color = Color.White,
        modifier = modifier.wrapContentWidth1(align = Alignment.End).wrapContentHeight()
    )
}

@Composable
private fun MtuBodyView(mtuValue: String, modifier: Modifier) {
    Row(modifier = modifier.wrapContentWidth1().wrapContentHeight()) {
        Text(
            text = mtuValue.ifEmpty { stringResource(id = R.string.hint_default) },
            color = Color.White
        )
    }
}

@Composable
private fun MtuSubtitle(modifier: Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_small).value.sp
    Text(
        text = stringResource(R.string.wireguard_mtu_footer, MTU_MIN_VALUE, MTU_MAX_VALUE),
        fontSize = textSize,
        color = MullvadWhite60,
        modifier = modifier
    )
}
