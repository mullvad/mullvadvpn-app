package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.material.Divider
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.layout.ContentScale
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
import net.mullvad.mullvadvpn.R

@Preview
@Composable
private fun PreviewDnsCell() {
    Column {
        DnsCell(
            dnsCellData = DnsCellData("35455", false),
            cellClick = {},
            onLostFocus = {},
            confirmClick = {},
            removeClick = null,
        )
        Divider()
        DnsCell(
            dnsCellData = DnsCellData("1.1.1.1", true),
            cellClick = {},
            onLostFocus = {},
            confirmClick = {},
            removeClick = null,
        )
        Divider()
        DnsCell(
            dnsCellData = DnsCellData(),
            cellClick = {},
            onLostFocus = {},
            confirmClick = {},
            removeClick = null,
        )
    }
}

@Composable
fun DnsCell(
    dnsCellData: DnsCellData,
    modifier: Modifier = Modifier,
    cellClick: () -> Unit,
    onLostFocus: () -> Unit,
    confirmClick: ((String) -> Unit)? = null,
    removeClick: (() -> Unit)? = null,
    onTextChanged: ((String) -> Unit) = {},
    validateInputDns: ((String) -> Boolean) = { true },
) {
    val focusRequester = remember { FocusRequester() }

    val cellHeight = dimensionResource(id = R.dimen.cell_height)
    val cellStartPadding = 54.dp
    val painterEndPadding = 6.dp
    val cellEndPadding = dimensionResource(id = R.dimen.side_margin)

    ConstraintLayout(
        modifier = modifier
            .height(cellHeight)
            .fillMaxWidth()
            .clickable { cellClick() }
            .background(colorResource(id = R.color.blue20))
    ) {
        val (title, icon) = createRefs()
        when {
            dnsCellData.isEditMode -> {
                DnsTextField(
                    value = dnsCellData.editValue,
                    modifier = Modifier
                        .focusRequester(focusRequester)
                        .fillMaxWidth()
                        .fillMaxHeight()
                        .background(colorResource(id = R.color.white))
                        .padding(start = 42.dp, end = cellStartPadding),
                    onValueChanged = { value -> onTextChanged(value) },
                    placeholderText = stringResource(id = R.string.custom_dns_hint),
                    onFocusChanges = { onLostFocus() },
                    onSubmit = { confirmClick?.invoke(dnsCellData.editValue) },
                    isEnabled = true,
                    maxCharLength = Int.MAX_VALUE,
                    isValidValue = { validateInputDns(it) }
                )

                LaunchedEffect(Unit) {
                    focusRequester.requestFocus()
                }
                Image(
                    painter = painterResource(id = R.drawable.icon_tick),
                    contentDescription = "Confirm DNS",
                    colorFilter = ColorFilter.tint(colorResource(id = R.color.green)),
                    contentScale = ContentScale.Inside,
                    modifier = Modifier
                        .constrainAs(icon) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        }
                        .padding(end = painterEndPadding)
                        .width(cellHeight)
                        .height(cellHeight)
                        .clickable { confirmClick?.invoke(dnsCellData.editValue) }
                )
            }
            dnsCellData.ip != null -> {
                Text(
                    text = dnsCellData.ip!!,
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
                    contentDescription = "remove DNS",
                    contentScale = ContentScale.Inside,
                    modifier = Modifier
                        .constrainAs(icon) {
                            top.linkTo(parent.top)
                            bottom.linkTo(parent.bottom)
                            end.linkTo(parent.end)
                        }
                        .padding(end = painterEndPadding)
                        .width(cellHeight)
                        .height(cellHeight)
                        .clickable { removeClick?.invoke() }
                )
            }
            else -> {
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

data class DnsCellData(
    var ip: String? = null,
    var isEditMode: Boolean = false,
    var editValue: String = "",

)
