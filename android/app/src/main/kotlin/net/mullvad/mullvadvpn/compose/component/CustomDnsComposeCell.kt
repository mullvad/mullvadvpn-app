package net.mullvad.mullvadvpn.compose.component

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.layout.wrapContentWidth
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.items
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
import net.mullvad.mullvadvpn.viewmodel.CellUiState
import java.net.InetAddress


@Preview
@Composable
fun CustomDnsComposeCellPreview() {
    CustomDnsComposeCell(
        isEnabled  = true,
        onToggle = {}
    )
}

@Composable
fun CustomDnsComposeCell(
    isEnabled: Boolean,
    onToggle: (Boolean) -> Unit
) {

    var titleModifier = Modifier
    var bodyViewModifier = Modifier
    var subtitleModifier = Modifier


    BaseCell(
        uiState = CellUiState.CustomDNSCellUiState(),
        title = { customDnsCellTitle(modifier = titleModifier) },
        titleModifier = titleModifier,
        bodyView = {
            customDnsCellView(
                isEnabled = isEnabled,
                switchTriggered = { newValue ->
                    onToggle(newValue)
                },
                modifier = bodyViewModifier
            )
        },
        bodyViewModifier = bodyViewModifier,
        expandableContent = {
//            dnsList()
        },
        expandableContentModifier = Modifier,
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
    isEnabled: Boolean,
    switchTriggered: (Boolean) -> Unit,
    modifier: Modifier
) {
    Row(
        modifier = modifier
            .wrapContentWidth()
            .wrapContentHeight()
    ) {
        // Declaring a boolean value for storing checked state
//        val mCheckedState = remember { mutableStateOf(defaultValue) }

        CellSwitch(
            checked = isEnabled,
            onCheckedChange = { newValue ->
//                mCheckedState.value = it
                switchTriggered(newValue)
            },
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




//@Composable
//fun customDnsCellExpandable(dnsList: List<InetAddress>?, expanded: Boolean, modifier: Modifier) {
//    Column(modifier = modifier) {
//        Divider()
//        AnimatedVisibility(
//            visible = expanded,
//        ) {
//            Column {
//
//                dnsList?.let {
//                    LazyColumn(
//                        modifier = Modifier
//                            .fillMaxWidth()
//                            .height(((dnsList.size * 53) + 52).dp)
//                    ) {
//
//                        items(it.size) { index ->
//                            DnsCell(DnsCellUiState(it[index]), { /*it.removeAt(index)*/ })
//                            Divider()
//                        }
//                        item {
//                            DnsCell(DnsCellUiState())
//                        }
//                    }
//                }
//            }
//        }
//    }
//}


//@Composable
fun customDnsCellExpandable(
    dnsList: List<String>,
    expanded: Boolean,
    modifier: Modifier,
    lazyListScope: LazyListScope
) {
//    Column(modifier = modifier) {
//        Divider()
//        AnimatedVisibility(
//            visible = expanded,
//        ) {
//            Column {

//                dnsList?.let {
//                    LazyColumn(
//                        modifier = Modifier
//                            .fillMaxWidth()
//                            .height(((dnsList.size * 53) + 52).dp)
//                    ) {
//
//                        items(it.size) { index ->
//
//                            Divider()
//                        }
//                        item {
//                            DnsCell(DnsCellUiState())
//                        }
//                    }

    lazyListScope.apply {
        items(items = dnsList) {
            DnsCell(DnsCellUiState(InetAddress.getByName(it)), { /*it.removeAt(index)*/ })
        }
    }
//                }
//            }
//        }
//    }
}
