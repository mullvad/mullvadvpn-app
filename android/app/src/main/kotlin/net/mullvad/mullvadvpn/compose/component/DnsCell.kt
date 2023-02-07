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
import androidx.compose.material.ButtonDefaults
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
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import java.net.InetAddress
import java.util.concurrent.locks.ReentrantReadWriteLock
import kotlin.concurrent.withLock
import kotlin.reflect.KClass

const val DefaultDnsValue = ""

@Preview
@Composable
fun PreviewDnsCell() {
    Column {
        DnsCell(DnsCellUiState())
        Divider()
        DnsCell(DnsCellUiState(InetAddress.getByName("35455")))
        Divider()
        DnsCell(DnsCellUiState(InetAddress.getByName("1.1.1.1"), true))
    }
}

@Composable
fun DnsCell(
    dnsCellUiState: DnsCellUiState,
    confirmClick: ((String) -> Unit)? = null,
    removeClick: (() -> Unit)? = null,
    modifier: Modifier = Modifier
) {

    var cellHeight = dimensionResource(id = R.dimen.cell_height)
    var cellStartPadding = 54.dp
    var cellEndPadding = dimensionResource(id = R.dimen.side_margin)

    ConstraintLayout(
        modifier = modifier
            .height(cellHeight)
            .fillMaxWidth()
    ) {
        val (title, icon, input) = createRefs()
        var cellMode = dnsCellUiState.dnsCellUiState.collectAsState().value
        when {
            cellMode.isInEditMode -> {
                var inputIp = dnsCellUiState.dnsCellUiState.collectAsState().value.ipString
                TextField(
                    value = inputIp,
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
                        .clickable { confirmClick?.invoke(inputIp) }
                )
            }
            cellMode is DnsCellUiState.DnsCellMode.NormalDns -> {
                Button(
                    onClick = {
                        dnsCellUiState.dnsCellUiState.value.isInEditMode = true
                    },
                    colors = ButtonDefaults.buttonColors(
                        backgroundColor = colorResource(id = android.R.color.transparent),
                        contentColor = MullvadBlue
                    ),
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
                ) {}
                    ConstraintLayout(
                        modifier = Modifier
                            .height(cellHeight)
                            .fillMaxWidth()
                    ) {
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

            }
            cellMode is DnsCellUiState.DnsCellMode.NewDns -> {
                Button(
                    onClick = {
                        dnsCellUiState.dnsCellUiState.value.isInEditMode = true
                    },
                    colors = ButtonDefaults.buttonColors(
                        backgroundColor = colorResource(id = android.R.color.transparent)
                    ),
                    modifier = Modifier
                        .fillMaxWidth()
                        .fillMaxHeight()
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
    val inputValue = MutableStateFlow<String>(ip?.hostAddress ?: "")

    init {
        dnsCellUiState.value = ip?.let {
            DnsCellMode.NormalDns(
                ipString = inputValue.value,
                ip = ip
            )

        } ?: kotlin.run { DnsCellMode.NewDns }
    }

    sealed interface DnsCellMode {
        var isInEditMode: Boolean
        var ipString: String

        data class NormalDns(
            override var ipString: String,
            var ip: InetAddress?,
        ) : DnsCellMode {
            private val _isInEditMode = MutableStateFlow(false)
            override var isInEditMode: Boolean
                get() = _isInEditMode.value
                set(value) {
                    lock.writeLock().withLock {
//                        currentEditModeDns?.let {
//                            it.isInEditMode = false
//                        }
                        _isInEditMode.value = value
                        currentEditModeDns = this
                    }
                }
        }

        object NewDns : DnsCellMode {
            private val _isInEditMode = MutableStateFlow(false)
            override var isInEditMode: Boolean
                get() = _isInEditMode.value
                set(value) {
                    lock.writeLock().withLock {
//                        currentEditModeDns?.let {
//                            it.isInEditMode = false
//                        }
                        _isInEditMode.value = value
                        currentEditModeDns = this
                    }
                }
            override var ipString: String = ""
        }


        companion object {
            var currentEditModeDns: DnsCellMode? = null
        }
    }
}
