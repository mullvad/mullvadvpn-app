package net.mullvad.mullvadvpn.compose.screen

import android.widget.Toast
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.Divider
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.LifecycleOwner
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.ContentBlockersDisableModeCellSubtitle
import net.mullvad.mullvadvpn.compose.cell.CustomDnsCellSubtitle
import net.mullvad.mullvadvpn.compose.cell.DnsCell
import net.mullvad.mullvadvpn.compose.cell.ExpandableComposeCell
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.MtuComposeCell
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SwitchCellTitle
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.dialog.ContentBlockersInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.CustomDnsInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.DnsDialog
import net.mullvad.mullvadvpn.compose.dialog.MalwareInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.MtuDialog
import net.mullvad.mullvadvpn.compose.dialog.ObfuscationInfoDialog
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.AdvancedSettingsUiState
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue20
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadGreen
import net.mullvad.mullvadvpn.model.SelectedObfuscation
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
                customDnsItems = listOf(CustomDnsItem("0.0.0.0", false)),
            ),
        onMtuCellClick = {},
        onMtuInputChange = {},
        onSaveMtuClick = {},
        onRestoreMtuClick = {},
        onCancelMtuDialogClicked = {},
        onSplitTunnelingNavigationClick = {},
        onToggleDnsClick = {},
        onToggleBlockAds = {},
        onToggleBlockTrackers = {},
        onToggleBlockMalware = {},
        onToggleBlockAdultContent = {},
        onToggleBlockGambling = {},
        onDnsClick = {},
        onDnsInputChange = {},
        onSaveDnsClick = {},
        onRemoveDnsClick = {},
        onCancelDnsDialogClick = {},
        onContentsBlockersInfoClicked = {},
        onMalwareInfoClicked = {},
        onCustomDnsInfoClicked = {},
        onDismissInfoClicked = {},
        onBackClick = {},
        toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow(),
        onStopEvent = {},
        onSelectObfuscationSetting = {},
        onObfuscationInfoClicked = {}
    )
}

@OptIn(ExperimentalFoundationApi::class)
@ExperimentalMaterialApi
@Composable
fun AdvancedSettingScreen(
    lifecycleOwner: LifecycleOwner = LocalLifecycleOwner.current,
    uiState: AdvancedSettingsUiState,
    onMtuCellClick: () -> Unit = {},
    onMtuInputChange: (String) -> Unit = {},
    onSaveMtuClick: () -> Unit = {},
    onRestoreMtuClick: () -> Unit = {},
    onCancelMtuDialogClicked: () -> Unit = {},
    onSplitTunnelingNavigationClick: () -> Unit = {},
    onToggleDnsClick: (Boolean) -> Unit = {},
    onToggleBlockAds: (Boolean) -> Unit = {},
    onToggleBlockTrackers: (Boolean) -> Unit = {},
    onToggleBlockMalware: (Boolean) -> Unit = {},
    onToggleBlockAdultContent: (Boolean) -> Unit = {},
    onToggleBlockGambling: (Boolean) -> Unit = {},
    onDnsClick: (index: Int?) -> Unit = {},
    onDnsInputChange: (String) -> Unit = {},
    onSaveDnsClick: () -> Unit = {},
    onRemoveDnsClick: () -> Unit = {},
    onCancelDnsDialogClick: () -> Unit = {},
    onContentsBlockersInfoClicked: () -> Unit = {},
    onMalwareInfoClicked: () -> Unit = {},
    onCustomDnsInfoClicked: () -> Unit = {},
    onDismissInfoClicked: () -> Unit = {},
    onBackClick: () -> Unit = {},
    onStopEvent: () -> Unit = {},
    toastMessagesSharedFlow: SharedFlow<String>,
    onSelectObfuscationSetting: (selectedObfuscation: SelectedObfuscation) -> Unit = {},
    onObfuscationInfoClicked: () -> Unit = {}
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
        is AdvancedSettingsUiState.ContentBlockersInfoDialogUiState -> {
            ContentBlockersInfoDialog(onDismissInfoClicked)
        }
        is AdvancedSettingsUiState.CustomDnsInfoDialogUiState -> {
            CustomDnsInfoDialog(onDismissInfoClicked)
        }
        is AdvancedSettingsUiState.MalwareInfoDialogUiState -> {
            MalwareInfoDialog(onDismissInfoClicked)
        }
        is AdvancedSettingsUiState.ObfuscationInfoDialogUiState -> {
            ObfuscationInfoDialog(onDismissInfoClicked)
        }
        else -> {
            // NOOP
        }
    }

    val lazyListState = rememberLazyListState()
    var expandContentBlockersState by rememberSaveable { mutableStateOf(false) }
    val biggerPadding = 54.dp
    val topPadding = 6.dp
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
        },
    ) {
        val context = LocalContext.current
        LaunchedEffect(Unit) {
            toastMessagesSharedFlow.distinctUntilChanged().collect { message ->
                Toast.makeText(context, message, Toast.LENGTH_SHORT).show()
            }
        }
        DisposableEffect(lifecycleOwner) {
            val observer = LifecycleEventObserver { _, event ->
                if (event == Lifecycle.Event.ON_STOP) {
                    onStopEvent()
                }
            }
            lifecycleOwner.lifecycle.addObserver(observer)
            onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
        }
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
                ExpandableComposeCell(
                    title = stringResource(R.string.dns_content_blockers_title),
                    isExpanded = !expandContentBlockersState,
                    isEnabled = !uiState.isCustomDnsEnabled,
                    onInfoClicked = { onContentsBlockersInfoClicked() },
                    onCellClicked = { expandContentBlockersState = !expandContentBlockersState }
                )
            }

            if (expandContentBlockersState) {
                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.block_ads_title),
                        isToggled = uiState.contentBlockersOptions.blockAds,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAds(it) },
                        background = MullvadBlue20
                    )
                }
                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.block_trackers_title),
                        isToggled = uiState.contentBlockersOptions.blockTrackers,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockTrackers(it) },
                        background = MullvadBlue20
                    )
                }
                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.block_malware_title),
                        isToggled = uiState.contentBlockersOptions.blockMalware,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockMalware(it) },
                        onInfoClicked = { onMalwareInfoClicked() },
                        background = MullvadBlue20
                    )
                }
                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.block_gambling_title),
                        isToggled = uiState.contentBlockersOptions.blockGambling,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockGambling(it) },
                        background = MullvadBlue20
                    )
                }
                itemWithDivider {
                    SwitchComposeCell(
                        title = stringResource(R.string.block_adult_content_title),
                        isToggled = uiState.contentBlockersOptions.blockAdultContent,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAdultContent(it) },
                        background = MullvadBlue20
                    )
                }

                if (uiState.isCustomDnsEnabled) {
                    item {
                        ContentBlockersDisableModeCellSubtitle(
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

            itemWithDivider {
                Spacer(modifier = Modifier.height(cellVerticalSpacing))
                InformationComposeCell(
                    title = stringResource(R.string.obfuscation_title),
                    onInfoClicked = { onObfuscationInfoClicked() },
                    onCellClicked = { expandContentBlockersState = !expandContentBlockersState }
                )
            }
            itemWithDivider {
                BaseCell(
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Auto) },
                    title = {
                        SwitchCellTitle(
                            title = stringResource(id = R.string.automatic),
                        )
                    },
                    background =
                        if (uiState.selectedObfuscation == SelectedObfuscation.Auto) MullvadGreen
                        else MullvadBlue20,
                )
            }
            itemWithDivider {
                BaseCell(
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Udp2Tcp) },
                    title = {
                        SwitchCellTitle(
                            title = stringResource(id = R.string.obfuscation_on_udp_over_tcp),
                        )
                    },
                    background =
                        if (uiState.selectedObfuscation == SelectedObfuscation.Udp2Tcp) MullvadGreen
                        else MullvadBlue20,
                )
            }
            itemWithDivider {
                BaseCell(
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Off) },
                    title = {
                        SwitchCellTitle(
                            title = stringResource(id = R.string.off),
                        )
                    },
                    background =
                        if (uiState.selectedObfuscation == SelectedObfuscation.Off) MullvadGreen
                        else MullvadBlue20,
                )
            }

            item {
                Spacer(modifier = Modifier.height(cellVerticalSpacing))
                SwitchComposeCell(
                    title = stringResource(R.string.enable_custom_dns),
                    isToggled = uiState.isCustomDnsEnabled,
                    isEnabled = uiState.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                    onInfoClicked = { onCustomDnsInfoClicked() }
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
                        startPadding = biggerPadding,
                    )
                }
            }

            item {
                CustomDnsCellSubtitle(
                    isCellClickable = uiState.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    modifier =
                        Modifier.background(MullvadDarkBlue)
                            .padding(
                                start = cellHorizontalSpacing,
                                top = topPadding,
                                end = cellHorizontalSpacing,
                                bottom = cellVerticalSpacing,
                            )
                )
            }
        }
    }
}
