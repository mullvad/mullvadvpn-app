package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.Divider
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.CustomDnsCellSubtitle
import net.mullvad.mullvadvpn.compose.cell.DnsCell
import net.mullvad.mullvadvpn.compose.cell.MtuComposeCell
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.DnsDialog
import net.mullvad.mullvadvpn.compose.dialog.MtuDialog
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.AdvancedSettingsUiState
import net.mullvad.mullvadvpn.compose.theme.CollapsingToolbarTheme
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue20
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

@OptIn(ExperimentalMaterialApi::class)
@Preview
@Composable
private fun PreviewAdvancedSettings() {
    AdvancedSettingScreen(
        uiState =
            AdvancedSettingsUiState.DefaultUiState(
                mtu = "1337",
                isCustomDnsEnabled = true,
                customDnsItems = listOf(CustomDnsItem("0.0.0.0", false))
            ),
        onMtuCellClick = {},
        onMtuInputChange = {},
        onSaveMtuClick = {},
        onRestoreMtuClick = {},
        onCancelMtuDialogClicked = {},
        onSplitTunnelingNavigationClick = {},
        onToggleDnsClick = {},
        onDnsClick = {},
        onDnsInputChange = {},
        onSaveDnsClick = {},
        onRemoveDnsClick = {},
        onCancelDnsDialogClick = {},
        onBackClick = {}
    )
}

@OptIn(ExperimentalFoundationApi::class)
@ExperimentalMaterialApi
@Composable
fun AdvancedSettingScreen(
    uiState: AdvancedSettingsUiState,
    onMtuCellClick: () -> Unit = {},
    onMtuInputChange: (String) -> Unit = {},
    onSaveMtuClick: () -> Unit = {},
    onRestoreMtuClick: () -> Unit = {},
    onCancelMtuDialogClicked: () -> Unit = {},
    onSplitTunnelingNavigationClick: () -> Unit = {},
    onToggleDnsClick: (Boolean) -> Unit = {},
    onDnsClick: (index: Int?) -> Unit = {},
    onDnsInputChange: (String) -> Unit = {},
    onSaveDnsClick: () -> Unit = {},
    onRemoveDnsClick: () -> Unit = {},
    onCancelDnsDialogClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val cellVerticalSpacing = dimensionResource(id = R.dimen.cell_label_vertical_padding)
    val cellHorizontalSpacing = dimensionResource(id = R.dimen.cell_left_padding)

    when (uiState) {
        is AdvancedSettingsUiState.MtuDialogUiState -> {
            MtuDialog(
                mtuValue = uiState.mtuEditValue,
                onMtuValueChanged = { onMtuInputChange(it) },
                onSave = { onSaveMtuClick() },
                onRestoreDefaultValue = { onRestoreMtuClick() },
                onDismiss = { onCancelMtuDialogClicked() }
            )
        }
        is AdvancedSettingsUiState.DnsDialogUiState -> {
            DnsDialog(
                stagedDns = uiState.stagedDns,
                isAllowLanEnabled = uiState.isAllowLanEnabled,
                onIpAddressChanged = { onDnsInputChange(it) },
                onAttemptToSave = { onSaveDnsClick() },
                onRemove = { onRemoveDnsClick() },
                onDismiss = { onCancelDnsDialogClick() }
            )
        }
        else -> {
            // NOOP
        }
    }

    val lazyListState = rememberLazyListState()
    val biggerPadding = 54.dp
    val topPadding = 6.dp

    CollapsingToolbarTheme {
        val state = rememberCollapsingToolbarScaffoldState()
        val progress = state.toolbarState.progress

        CollapsableAwareToolbarScaffold(
            backgroundColor = MullvadDarkBlue,
            modifier = Modifier.fillMaxSize(),
            state = state,
            scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
            isEnabledWhenCollapsable = true,
            toolbar = {
                val scaffoldModifier =
                    Modifier.road(
                        whenCollapsed = Alignment.TopCenter,
                        whenExpanded = Alignment.BottomStart
                    )
                CollapsingTopBar(
                    backgroundColor = MullvadDarkBlue,
                    onBackClicked = { onBackClick() },
                    title = stringResource(id = R.string.settings_advanced),
                    progress = progress,
                    modifier = scaffoldModifier,
                    backTitle = stringResource(id = R.string.settings)
                )
            }
        ) {
            LazyColumn(
                modifier =
                    Modifier.drawVerticalScrollbar(lazyListState)
                        .fillMaxWidth()
                        .wrapContentHeight()
                        .animateContentSize(),
                state = lazyListState
            ) {
                item { MtuComposeCell(mtuValue = uiState.mtu, onEditMtu = { onMtuCellClick() }) }

                itemWithDivider {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.split_tunneling),
                        onClick = { onSplitTunnelingNavigationClick.invoke() }
                    )
                }

                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.enable_custom_dns),
                        isEnabled = true,
                        isToggled = uiState.isCustomDnsEnabled,
                        onCellClicked = { newValue -> onToggleDnsClick(newValue) }
                    )
                }

                if (uiState.isCustomDnsEnabled) {
                    itemsIndexed(uiState.customDnsItems) { index, item ->
                        DnsCell(
                            address = item.address,
                            isUnreachableLocalDnsWarningVisible =
                                item.isLocal && uiState.isAllowLanEnabled.not(),
                            onClick = { onDnsClick(index) },
                            modifier = Modifier.animateItemPlacement()
                        )
                        Divider()
                    }

                    itemWithDivider {
                        BaseCell(
                            onCellClicked = { onDnsClick(null) },
                            title = {
                                Text(
                                    text = stringResource(id = R.string.add_a_server),
                                    color = Color.White,
                                )
                            },
                            bodyView = {},
                            subtitle = null,
                            background = MullvadBlue20,
                            startPadding = biggerPadding
                        )
                    }
                }

                item {
                    CustomDnsCellSubtitle(
                        isCellClickable = true,
                        modifier =
                            Modifier.background(MullvadDarkBlue)
                                .padding(
                                    start = cellHorizontalSpacing,
                                    top = topPadding,
                                    end = cellHorizontalSpacing,
                                    bottom = cellVerticalSpacing
                                )
                    )
                }
            }
        }
    }
}
