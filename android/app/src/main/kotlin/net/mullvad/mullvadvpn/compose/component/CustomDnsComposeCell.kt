package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.Divider
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.viewmodel.CellUiState

@Preview
@Composable
fun CustomDnsComposeCellPreview() {
    CustomDnsComposeCell(
        checkboxDefaultState = true,
        onToggle = {},
        dnsList = emptyList()
    )
}

@Composable
fun CustomDnsComposeCell(
    checkboxDefaultState: Boolean,
    onToggle: (Boolean) -> Unit,
    dnsList: List<String>
) {

    var titleModifier = Modifier
    var bodyViewModifier = Modifier
    var expandableViewModifier = Modifier
    var subtitleModifier = Modifier
    var expanded by remember { mutableStateOf(checkboxDefaultState) }

    val rotateState = animateFloatAsState(
        targetValue = if (expanded) 180F else 0F,
    )

    BaseCell(
        uiState = CellUiState.CustomDNSCellUiState(),
        title = { customDnsCellTitle(modifier = titleModifier) },
        titleModifier = titleModifier,
        bodyView = {
            customDnsCellView(
                switchTriggered = {
                    onToggle(it)
                    expanded = it
                },
                defaultValue = expanded,
                modifier = bodyViewModifier
            )
        },
        bodyViewModifier = bodyViewModifier,
        expandableContent = {
            customDnsCellExpandable(
                dnsList = dnsList,
                expanded = expanded,
                modifier = expandableViewModifier
            )
        },
        expandableContentModifier = Modifier,
        subtitle = { customDnsCellSubtitle(subtitleModifier) },
        subtitleModifier = subtitleModifier
    )
}

@Composable
fun customDnsCellTitle(
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
fun customDnsCellView(
    switchTriggered: (Boolean) -> Unit,
    defaultValue: Boolean = false,
    modifier: Modifier
) {
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight()
    ) {
        // Declaring a boolean value for storing checked state
        val mCheckedState = remember { mutableStateOf(defaultValue) }

        CellSwitch(
            checked = mCheckedState.value,
            onCheckedChange = {
                mCheckedState.value = it
                switchTriggered(it)
            },
        )
    }
}

@Composable
fun customDnsCellExpandable(dnsList: List<String>?, expanded: Boolean, modifier: Modifier) {
    Column(modifier = modifier) {
        Divider()
        AnimatedVisibility(
            visible = expanded,
        ) {
            Column {

                dnsList?.let {
                    LazyColumn(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(((dnsList.size * 53) + 52).dp)
                    ) {

                        items(it.size) { index ->
                            DnsCell(DnsCellUiState(it[index]), { /*it.removeAt(index)*/ })
                            Divider()
                        }
                        item {
                            DnsCell(DnsCellUiState())
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun customDnsCellSubtitle(modifier: Modifier) {
    Text(
        text = stringResource(R.string.custom_dns_footer),
        fontSize = 13.sp,
        color = colorResource(id = R.color.white60),
        modifier = modifier

    )
}
