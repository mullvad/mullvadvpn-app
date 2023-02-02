package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth as wrapContentWidth1
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.viewmodel.CellUiState

private const val MIN_MTU_VALUE = 1280
private const val MAX_MTU_VALUE = 1420

@Preview
@Composable
fun MtuComposeCellPreview() {
//    MtuComposeCell("1300", {})
}

var tmpp = MutableStateFlow("")

@Preview
@Composable
fun contentPreview() {
    mtuBodyView(
        tmpp.collectAsState().value,
        { tmpp.value = it },
        { tmpp.value = it },
        Modifier,
    )
}

@Composable
fun MtuComposeCell(
    mtuValue: String?,
    onMtuChanged: (String) -> Unit,
    onMtuSubmit: (String) -> Unit
) {
    var titleModifier = Modifier
    var rightViewModifier = Modifier
    var subtitleModifier = Modifier

    Column() {
//        var currentMtu = value.uiState.collectAsState().value.mtuState.mtuValue?.wireguardMtu
//        var mtuString: String = currentMtu?.let { it.toString() } ?: run{ "" }
        BaseCell(
            uiState = CellUiState.MTUCellUiState(),
            title = { mtuTitle(modifier = titleModifier) },
            titleModifier = titleModifier,
            bodyView = {
                mtuBodyView(
                    mtu = mtuValue ?: "",
                    onMtuChanged = { onMtuChanged.invoke(it) },
                    onMtuSubmit = onMtuSubmit,
                    modifier = titleModifier
                )
            },
            bodyViewModifier = rightViewModifier,
            subtitle = { mtuSubtitle(subtitleModifier) },
            subtitleModifier = subtitleModifier
        )
    }
}

@Composable
fun mtuTitle(
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
fun mtuBodyView(
    mtu: String,
    onMtuChanged: (String) -> Unit,
    onMtuSubmit: (String) -> Unit,
    modifier: Modifier
) {
    Row(
        modifier = modifier
            .wrapContentWidth1()
            .wrapContentHeight()
    ) {
        TextField(
            value = mtu,
            singleLine = true,
            onValueChange = { onMtuChanged(it) },
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
            keyboardActions = KeyboardActions(
                onDone = { onMtuSubmit(mtu) }
            ),
            placeholder = {
                Text(
                    text = stringResource(id = R.string.hint_default),
                    color = colorResource(
                        id = R.color.blue
                    )
                )
            },
            modifier = Modifier
                .onFocusChanged {
                    if (it.isFocused) {
                        // focused
                    } else {
                        // not focused
                        onMtuSubmit(mtu)
                    }
                }
                .width(96.dp)
                .wrapContentHeight()
                .defaultMinSize(minHeight = 64.dp)
                .padding(top = 0.dp, bottom = 0.dp)
                .background(colorResource(id = R.color.white10), shape = RoundedCornerShape(4.dp)),
            colors = TextFieldDefaults.textFieldColors(
                backgroundColor = colorResource(id = R.color.white10),
                focusedIndicatorColor = Color.Transparent, // hide the indicator
                unfocusedIndicatorColor = colorResource(id = R.color.white20),
                textColor = colorResource(id = R.color.white),

            )

        )
    }
}

@Composable
fun mtuSubtitle(modifier: Modifier) {
    Text(
        text = stringResource(R.string.wireguard_mtu_footer, MIN_MTU_VALUE, MAX_MTU_VALUE),
        fontSize = 13.sp,
        color = colorResource(id = R.color.white60),
        modifier = modifier

    )
}
