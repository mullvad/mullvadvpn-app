package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Icon
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.theme.MullvadHelmetYellow

@Preview
@Composable
private fun PreviewDnsCell() {
    DnsCell(address = "0.0.0.0", isUnreachableLocalDnsWarningVisible = true, onClick = {})
}

@Composable
fun DnsCell(
    address: String,
    isUnreachableLocalDnsWarningVisible: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    val titleModifier = Modifier
    val startPadding = 54.dp

    BaseCell(
        title = { DnsTitle(address = address, modifier = titleModifier) },
        bodyView = {
            if (isUnreachableLocalDnsWarningVisible) {
                Icon(
                    painter = painterResource(id = R.drawable.icon_alert),
                    contentDescription = stringResource(id = R.string.confirm_local_dns),
                    tint = MullvadHelmetYellow
                )
            }
        },
        onCellClicked = { onClick.invoke() },
        background = colorResource(id = R.color.blue20),
        startPadding = startPadding,
        modifier = modifier
    )
}

@Composable
private fun DnsTitle(address: String, modifier: Modifier = Modifier) {
    val textSize = dimensionResource(id = R.dimen.text_medium).value.sp
    Text(
        text = address,
        color = Color.White,
        fontSize = textSize,
        fontStyle = FontStyle.Normal,
        textAlign = TextAlign.Start,
        modifier = modifier.wrapContentWidth(align = Alignment.End).wrapContentHeight()
    )
}
