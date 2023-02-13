package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewDnsComposeCell() {
    CustomDnsComposeCell(
        checkboxDefaultState = true,
        onToggle = {}
    )
}

@Composable
fun CustomDnsComposeCell(
    checkboxDefaultState: Boolean,
    onToggle: (Boolean) -> Unit
) {
    val titleModifier = Modifier
    val bodyViewModifier = Modifier
    val subtitleModifier = Modifier

    BaseCell(
        title = { CustomDnsCellTitle(modifier = titleModifier) },
        bodyView = {
            CustomDnsCellView(
                switchTriggered = {
                    onToggle(it)
                },
                isToggled = checkboxDefaultState,
                modifier = bodyViewModifier
            )
        },
        onCellClicked = { onToggle(!checkboxDefaultState) },
        subtitleModifier = subtitleModifier
    )
}

@Composable
fun CustomDnsCellTitle(
    modifier: Modifier
) {
    Text(
        text = stringResource(R.string.enable_custom_dns),
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = 18.sp,
        color = Color.White,
        modifier = modifier
            .wrapContentWidth(align = Alignment.End)
            .wrapContentHeight()
    )
}

@Composable
fun CustomDnsCellView(
    switchTriggered: (Boolean) -> Unit,
    isToggled: Boolean,
    modifier: Modifier
) {
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight()
    ) {
        CellSwitch(
            checked = isToggled,
            onCheckedChange = {
                switchTriggered(it)
            }
        )
    }
}

@Composable
fun CustomDnsCellSubtitle(modifier: Modifier) {
    Text(
        text = stringResource(R.string.custom_dns_footer),
        fontSize = 13.sp,
        color = colorResource(id = R.color.white60),
        modifier = modifier
    )
}
