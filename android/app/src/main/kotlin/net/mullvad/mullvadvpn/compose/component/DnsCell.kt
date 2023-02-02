package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Button
import androidx.compose.material.Divider
import androidx.compose.material.Text
import androidx.compose.material.TextField
import androidx.compose.material.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.R

const val DefaultDnsValue = ""

@Preview
@Composable
fun PreviewDnsCell() {
    Column {
        DnsCell(DnsCellUiState())
        Divider()
        DnsCell(DnsCellUiState("35455"))
        Divider()
        DnsCell(DnsCellUiState("1.1.1.1", true))
    }
}

@Composable
fun DnsCell(
    dnsCellUiState: DnsCellUiState,
    modifier: Modifier = Modifier,
    removeClick: (() -> Unit)? = null,
) {

    val cellHeight = dimensionResource(id = R.dimen.cell_height)
    val cellStartPadding = 54.dp
    val cellEndPadding = dimensionResource(id = R.dimen.side_margin)

    ConstraintLayout(
        modifier = modifier
            .height(cellHeight)
            .fillMaxWidth()
    ) {
        val (title, icon) = createRefs()
        when (val cellMode = dnsCellUiState.dnsCellUiState.collectAsState().value) {
            is DnsCellUiState.DnsCEllMode.NormalDns -> {
                Button(
                    onClick = {
                        dnsCellUiState.dnsCellUiState.value = DnsCellUiState.DnsCEllMode
                            .EditModeDns(cellMode.ip)
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.blue20))
                ) {
                }

                Text(
                    text = cellMode.ip,
                    color = colorResource(id = R.color.white),
                    fontSize = 16.sp,
                    fontStyle = FontStyle.Normal,
                    textAlign = TextAlign.Start,
                    modifier = Modifier
                        .constrainAs(title) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            start.linkTo(parent.start)
                        }
                        .fillMaxWidth()
                        .padding(start = cellStartPadding, top = 14.dp, bottom = 14.dp)
                )

                Image(
                    painter = painterResource(id = R.drawable.icon_close),
                    contentDescription = null,
                    modifier = Modifier
                        .constrainAs(icon) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        }
                        .padding(end = cellEndPadding)
                        .wrapContentWidth()
                        .wrapContentHeight()
                        .clickable { removeClick?.invoke() }
                )
            }
            is DnsCellUiState.DnsCEllMode.EditModeDns -> {

                TextField(
                    value = cellMode.ip,
//                    value = "",
                    onValueChange = {},
                    placeholder = {
                        Text(
                            text = stringResource(id = R.string.hint_default),
                            color = colorResource(id = R.color.blue60),
                            fontSize = 16.sp,
                            fontStyle = FontStyle.Normal,
                            textAlign = TextAlign.Start,
                            modifier = Modifier
                                .constrainAs(title) {
                                    top.linkTo(parent.top)
                                    bottom.linkTo(parent.bottom)
                                    start.linkTo(parent.start)
                                }
                                .fillMaxWidth()
                                .padding(start = cellStartPadding, top = 14.dp, bottom = 14.dp)
                        )
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.white))
                        .padding(start = cellStartPadding, end = cellStartPadding),
                    colors = TextFieldDefaults.textFieldColors(
                        backgroundColor = colorResource(id = R.color.white),
                        focusedIndicatorColor = Color.Black, // hide the indicator
                        unfocusedIndicatorColor = colorResource(id = R.color.white20),
//                        textColor = colorResource(id = R.color.bl),

                    ),
//
                )

                Image(
                    painter = painterResource(id = R.drawable.icon_tick),
                    contentDescription = null,
                    colorFilter = ColorFilter.tint(colorResource(id = R.color.green)),
                    modifier = Modifier
                        .constrainAs(icon) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        }
                        .padding(end = cellEndPadding)
                        .wrapContentWidth()
                        .wrapContentHeight()
                        .clickable { removeClick?.invoke() }
                )
            }
            is DnsCellUiState.DnsCEllMode.NewDns -> {
                Button(
                    onClick = {
                        dnsCellUiState.dnsCellUiState.value = DnsCellUiState.DnsCEllMode
                            .EditModeDns(DefaultDnsValue)
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.blue80))
                ) {
                }

                Text(
                    text = stringResource(id = R.string.add_a_server),
                    color = colorResource(id = R.color.white),
                    fontSize = 16.sp,
                    fontStyle = FontStyle.Normal,
                    textAlign = TextAlign.Start,
                    modifier = Modifier
                        .constrainAs(title) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            start.linkTo(parent.start)
                        }
                        .fillMaxWidth()
                        .padding(start = cellStartPadding, top = 14.dp, bottom = 14.dp)
                )

                Image(
                    painter = painterResource(id = R.drawable.ic_icons_add),
                    contentDescription = null,
                    colorFilter = ColorFilter.tint(colorResource(id = R.color.white60)),
                    modifier = Modifier
                        .constrainAs(icon) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        }
                        .padding(end = cellEndPadding)
                        .wrapContentWidth()
                        .wrapContentHeight()
                )
            }
        }
    }
}

class DnsCellUiState(var ip: String? = null, var isEditMode: Boolean = false) {
    val dnsCellUiState = MutableStateFlow<DnsCEllMode>(DnsCEllMode.NewDns)

    init {
        ip?.let {
            dnsCellUiState.value =
                if (isEditMode) {
                    DnsCEllMode.EditModeDns(it)
                } else {
                    DnsCEllMode.NormalDns(it)
                }
        }
    }

    sealed class DnsCEllMode {
        data class NormalDns(var ip: String) : DnsCEllMode()
        data class EditModeDns(var ip: String) : DnsCEllMode()
        object NewDns : DnsCEllMode()
    }
}
