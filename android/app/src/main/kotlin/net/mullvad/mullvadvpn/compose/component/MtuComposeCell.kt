package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth as wrapContentWidth1
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

@Preview
@Composable
fun MtuComposeCellPreview() {
    MtuComposeCell(
        mtuValue = "1300",
        onMtuChanged = {},
        onMtuSubmit = {},
        onMtuFocusChanged = {}
    )
}

@Composable
fun MtuComposeCell(
    mtuValue: String?,
    onMtuChanged: (String) -> Unit,
    onMtuSubmit: (String) -> Unit,
    onMtuFocusChanged: (Boolean) -> Unit,
) {
    val titleModifier = Modifier
    val subtitleModifier = Modifier

    val inputFocusRequester = remember { FocusRequester() }

    BaseCell(
        title = { MtuTitle(modifier = titleModifier) },
        bodyView = {
            MtuBodyView(
                mtuValue = mtuValue ?: "",
                onMtuChanged = { onMtuChanged.invoke(it) },
                onMtuSubmit = onMtuSubmit,
                onMtuFocusChanged = onMtuFocusChanged,
                modifier = titleModifier,
                inputFocusRequester = inputFocusRequester
            )
        },
        subtitle = { MtuSubtitle(subtitleModifier) },
        subtitleModifier = subtitleModifier,
        onCellClicked = {
            inputFocusRequester.requestFocus()
        }
    )
}

@Composable
private fun MtuTitle(
    modifier: Modifier
) {
    Text(
        text = stringResource(R.string.wireguard_mtu),
        textAlign = TextAlign.Center,
        fontWeight = FontWeight.Bold,
        fontSize = 18.sp,
        color = Color.White,
        modifier = modifier
            .wrapContentWidth1(align = Alignment.End)
            .wrapContentHeight()
    )
}

@Composable
private fun MtuBodyView(
    mtuValue: String,
    onMtuChanged: (String) -> Unit,
    onMtuSubmit: (String) -> Unit,
    onMtuFocusChanged: (Boolean) -> Unit,
    modifier: Modifier,
    inputFocusRequester: FocusRequester
) {
    Row(
        modifier = modifier
            .wrapContentWidth1()
            .wrapContentHeight()
    ) {
        CellTextField(
            value = mtuValue,
            onValueChanged = { newMtuValue ->
                onMtuChanged(newMtuValue)
            },
            onFocusChanges = {
                onMtuFocusChanged(it)
            },
            onSubmit = { newMtuValue ->
                onMtuSubmit(newMtuValue)
            },
            isEnabled = true,
            placeholderText = stringResource(id = R.string.hint_default),
            maxCharLength = 4,
            isValidValue = { return@CellTextField it.toIntOrNull() in 1280..1420 },
            inputFocusRequester = inputFocusRequester
        )
    }
}

@Composable
private fun MtuSubtitle(modifier: Modifier) {
    Text(
        text = stringResource(R.string.wireguard_mtu_footer, MIN_MTU_VALUE, MAX_MTU_VALUE),
        fontSize = 13.sp,
        color = colorResource(id = R.color.white60),
        modifier = modifier
    )
}
