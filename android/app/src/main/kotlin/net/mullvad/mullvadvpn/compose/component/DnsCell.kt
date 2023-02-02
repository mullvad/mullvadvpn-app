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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.remember
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.TextRange
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.constraintlayout.compose.ConstraintLayout
import kotlinx.coroutines.flow.MutableStateFlow
import net.mullvad.mullvadvpn.R
import java.net.InetAddress
import java.util.concurrent.locks.ReentrantReadWriteLock

const val DefaultDnsValue = ""

enum class CursorSelectionBehaviour {
    START, END, SELECT_ALL
}

@Preview
@Composable
fun PreviewDnsCell() {
    Column {
        DnsCell(
            dnsCellUiState = DnsCellUiState(),
            cellClick = {},
            confirmClick = {},
            removeClick = null,
        )
        Divider()
        DnsCell(
            dnsCellUiState = DnsCellUiState(InetAddress.getByName("35455"), false),
            cellClick = {},
            confirmClick = {},
            removeClick = null,
        )
        Divider()
        DnsCell(
            dnsCellUiState = DnsCellUiState(InetAddress.getByName("1.1.1.1"), true),
            cellClick = {},
            confirmClick = {},
            removeClick = null,
        )
    }
}

@Composable
fun DnsCell(
    dnsCellUiState: DnsCellUiState,
    modifier: Modifier = Modifier,
    cellClick: () -> Unit,
    confirmClick: ((String) -> Unit)? = null,
    removeClick: (() -> Unit)? = null,
    onTextChanged: ((String) -> Unit)? = null,
    behaviour: CursorSelectionBehaviour = CursorSelectionBehaviour.END,
) {
    val focusRequester = remember { FocusRequester() }

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
            is DnsCellUiState.DnsCellMode.EditDns -> {
                var inputIp = cellMode.newIpString
                val direction = LocalLayoutDirection.current
                var tfv by remember {
                    val selection = when (behaviour) {
                        CursorSelectionBehaviour.START -> {
                            if (direction == LayoutDirection.Ltr) {
                                TextRange.Zero
                            } else {
                                TextRange(inputIp.length)
                            }
                        }
                        CursorSelectionBehaviour.END -> {
                            if (direction == LayoutDirection.Ltr) TextRange(inputIp.length) else TextRange.Zero
                        }
                        CursorSelectionBehaviour.SELECT_ALL -> TextRange(0, inputIp.length)
                    }
                    val textFieldValue = TextFieldValue(text = inputIp, selection = selection)
                    mutableStateOf(textFieldValue)
                }


                TextField(
                    value = cellMode.newIpString,
                    onValueChange = { value -> onTextChanged?.invoke(value) },
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
                                .padding(
                                    start = cellStartPadding,
                                    top = 14.dp,
                                    bottom = 14.dp
                                )
                        )
                    },
                    modifier = Modifier
                        .focusRequester(focusRequester)
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.white))
                        .padding(start = cellStartPadding, end = cellStartPadding),
                    colors = TextFieldDefaults.textFieldColors(
                        backgroundColor = colorResource(id = R.color.white),
                        focusedIndicatorColor = Color.Black, // hide the indicator
                        unfocusedIndicatorColor = colorResource(id = R.color.white20),

                        ),
//
                )

                LaunchedEffect(Unit) {
                    focusRequester.requestFocus()
                }

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
                        .clickable { confirmClick?.invoke(cellMode.newIpString) }
                )
            }
            is DnsCellUiState.DnsCellMode.NormalDns -> {
                Button(
                    onClick = {
                        cellClick()
                    },
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.blue20))
                ) {
                }

                Text(
                    text = cellMode.ipString,
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
            is DnsCellUiState.DnsCellMode.NewDns -> {
                Button(
                    onClick = {
                        cellClick()
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

var lock = ReentrantReadWriteLock()

class DnsCellUiState(var ip: InetAddress? = null, var isEditMode: Boolean = false) {
    val dnsCellUiState = MutableStateFlow<DnsCellMode>(DnsCellMode.NewDns)
    val inputValue = MutableStateFlow(ip?.hostAddress ?: "")

    init {
        dnsCellUiState.value = when {
            isEditMode -> DnsCellMode.EditDns(
                ipString = inputValue.value,
                newIpString = ip?.hostAddress ?: ""
            )
            ip == null -> DnsCellMode.NewDns
            else -> DnsCellMode.NormalDns(
                ipString = inputValue.value
            )

        }
    }

    sealed interface DnsCellMode {
        var ipString: String

        data class NormalDns(override var ipString: String) : DnsCellMode
        data class EditDns(override var ipString: String, var newIpString: String) :
            DnsCellMode

        object NewDns : DnsCellMode {
            private var localIpString: String = ""

            override var ipString: String
                get() = localIpString
                set(value) {
                    localIpString = value
                }
        }

    }
}
