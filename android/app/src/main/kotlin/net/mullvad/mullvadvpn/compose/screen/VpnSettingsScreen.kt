package net.mullvad.mullvadvpn.compose.screen

import android.widget.Toast
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLifecycleOwner
import androidx.compose.ui.platform.testTag
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
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.ContentBlockersDisableModeCellSubtitle
import net.mullvad.mullvadvpn.compose.cell.CustomDnsCellSubtitle
import net.mullvad.mullvadvpn.compose.cell.CustomPortCell
import net.mullvad.mullvadvpn.compose.cell.DnsCell
import net.mullvad.mullvadvpn.compose.cell.ExpandableComposeCell
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.InformationComposeCell
import net.mullvad.mullvadvpn.compose.cell.MtuComposeCell
import net.mullvad.mullvadvpn.compose.cell.MtuSubtitle
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.NormalSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.ContentBlockersInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.CustomDnsInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.CustomPortDialog
import net.mullvad.mullvadvpn.compose.dialog.DnsDialog
import net.mullvad.mullvadvpn.compose.dialog.LocalNetworkSharingInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.MalwareInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.MtuDialog
import net.mullvad.mullvadvpn.compose.dialog.ObfuscationInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.QuantumResistanceInfoDialog
import net.mullvad.mullvadvpn.compose.dialog.WireguardPortInfoDialog
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.VpnSettingsDialog
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.util.hasValue
import net.mullvad.mullvadvpn.util.isCustom
import net.mullvad.mullvadvpn.util.toDisplayCustomPort
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem

@Preview
@Composable
private fun PreviewVpnSettings() {
    AppTheme {
        VpnSettingsScreen(
            uiState =
                VpnSettingsUiState.createDefault(
                    isAutoConnectEnabled = true,
                    mtu = "1337",
                    isCustomDnsEnabled = true,
                    customDnsItems = listOf(CustomDnsItem("0.0.0.0", false)),
                ),
            onMtuCellClick = {},
            onSaveMtuClick = {},
            onRestoreMtuClick = {},
            onCancelMtuDialogClick = {},
            onToggleAutoConnect = {},
            onToggleLocalNetworkSharing = {},
            onToggleDnsClick = {},
            onToggleBlockAds = {},
            onToggleBlockTrackers = {},
            onToggleBlockMalware = {},
            onToggleBlockAdultContent = {},
            onToggleBlockGambling = {},
            onToggleBlockSocialMedia = {},
            onDnsClick = {},
            onDnsInputChange = {},
            onSaveDnsClick = {},
            onRemoveDnsClick = {},
            onCancelDnsDialogClick = {},
            onLocalNetworkSharingInfoClick = {},
            onContentsBlockersInfoClick = {},
            onMalwareInfoClick = {},
            onCustomDnsInfoClick = {},
            onDismissInfoClick = {},
            onBackClick = {},
            toastMessagesSharedFlow = MutableSharedFlow<String>().asSharedFlow(),
            onStopEvent = {},
            onSelectObfuscationSetting = {},
            onObfuscationInfoClick = {},
            onSelectQuantumResistanceSetting = {},
            onQuantumResistanceInfoClicked = {},
            onWireguardPortSelected = {},
            onWireguardPortInfoClicked = {},
            onShowCustomPortDialog = {},
            onCancelCustomPortDialogClick = {},
            onCloseCustomPortDialog = {}
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun VpnSettingsScreen(
    lifecycleOwner: LifecycleOwner = LocalLifecycleOwner.current,
    uiState: VpnSettingsUiState,
    onAutoConnectClick: () -> Unit = {},
    onMtuCellClick: () -> Unit = {},
    onSaveMtuClick: (Int) -> Unit = {},
    onRestoreMtuClick: () -> Unit = {},
    onCancelMtuDialogClick: () -> Unit = {},
    onToggleAutoConnect: (Boolean) -> Unit = {},
    onToggleLocalNetworkSharing: (Boolean) -> Unit = {},
    onToggleDnsClick: (Boolean) -> Unit = {},
    onToggleBlockAds: (Boolean) -> Unit = {},
    onToggleBlockTrackers: (Boolean) -> Unit = {},
    onToggleBlockMalware: (Boolean) -> Unit = {},
    onToggleBlockAdultContent: (Boolean) -> Unit = {},
    onToggleBlockGambling: (Boolean) -> Unit = {},
    onToggleBlockSocialMedia: (Boolean) -> Unit = {},
    onDnsClick: (index: Int?) -> Unit = {},
    onDnsInputChange: (String) -> Unit = {},
    onSaveDnsClick: () -> Unit = {},
    onRemoveDnsClick: () -> Unit = {},
    onCancelDnsDialogClick: () -> Unit = {},
    onLocalNetworkSharingInfoClick: () -> Unit = {},
    onContentsBlockersInfoClick: () -> Unit = {},
    onMalwareInfoClick: () -> Unit = {},
    onCustomDnsInfoClick: () -> Unit = {},
    onDismissInfoClick: () -> Unit = {},
    onBackClick: () -> Unit = {},
    onStopEvent: () -> Unit = {},
    toastMessagesSharedFlow: SharedFlow<String>,
    onSelectObfuscationSetting: (selectedObfuscation: SelectedObfuscation) -> Unit = {},
    onObfuscationInfoClick: () -> Unit = {},
    onSelectQuantumResistanceSetting: (quantumResistant: QuantumResistantState) -> Unit = {},
    onQuantumResistanceInfoClicked: () -> Unit = {},
    onWireguardPortSelected: (port: Constraint<Port>) -> Unit = {},
    onWireguardPortInfoClicked: () -> Unit = {},
    onShowCustomPortDialog: () -> Unit = {},
    onCancelCustomPortDialogClick: () -> Unit = {},
    onCloseCustomPortDialog: () -> Unit = {}
) {
    val savedCustomPort = rememberSaveable { mutableStateOf<Constraint<Port>>(Constraint.Any()) }

    when (val dialog = uiState.dialog) {
        is VpnSettingsDialog.Mtu -> {
            MtuDialog(
                mtuInitial = dialog.mtuEditValue.toIntOrNull(),
                onSave = { onSaveMtuClick(it) },
                onRestoreDefaultValue = { onRestoreMtuClick() },
                onDismiss = { onCancelMtuDialogClick() }
            )
        }
        is VpnSettingsDialog.Dns -> {
            DnsDialog(
                stagedDns = dialog.stagedDns,
                isAllowLanEnabled = uiState.isAllowLanEnabled,
                onIpAddressChanged = { onDnsInputChange(it) },
                onAttemptToSave = { onSaveDnsClick() },
                onRemove = { onRemoveDnsClick() },
                onDismiss = { onCancelDnsDialogClick() }
            )
        }
        is VpnSettingsDialog.LocalNetworkSharingInfo -> {
            LocalNetworkSharingInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.ContentBlockersInfo -> {
            ContentBlockersInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.CustomDnsInfo -> {
            CustomDnsInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.MalwareInfo -> {
            MalwareInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.ObfuscationInfo -> {
            ObfuscationInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.QuantumResistanceInfo -> {
            QuantumResistanceInfoDialog(onDismissInfoClick)
        }
        is VpnSettingsDialog.WireguardPortInfo -> {
            WireguardPortInfoDialog(dialog.availablePortRanges, onDismissInfoClick)
        }
        is VpnSettingsDialog.CustomPort -> {
            CustomPortDialog(
                customPort = savedCustomPort.value.toDisplayCustomPort(),
                allowedPortRanges = dialog.availablePortRanges,
                onSave = { customPortString ->
                    onWireguardPortSelected(Constraint.Only(Port(customPortString.toInt())))
                },
                onReset = {
                    if (uiState.selectedWireguardPort.isCustom()) {
                        onWireguardPortSelected(Constraint.Any())
                    }
                    savedCustomPort.value = Constraint.Any()
                    onCloseCustomPortDialog()
                },
                showReset = savedCustomPort.value is Constraint.Only,
                onDismissRequest = { onCancelCustomPortDialogClick() }
            )
        }
    }

    var expandContentBlockersState by rememberSaveable { mutableStateOf(false) }
    val biggerPadding = 54.dp
    val topPadding = 6.dp

    LaunchedEffect(uiState.selectedWireguardPort) {
        if (
            uiState.selectedWireguardPort.isCustom() &&
                uiState.selectedWireguardPort != savedCustomPort.value
        ) {
            savedCustomPort.value = uiState.selectedWireguardPort
        }
    }

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
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_vpn),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_TEST_TAG).animateContentSize(),
            state = lazyListState
        ) {
            item {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                NavigationComposeCell(
                    title = stringResource(id = R.string.auto_connect_and_lockdown_mode),
                    onClick = { onAutoConnectClick() },
                )
            }
            item {
                SwitchComposeSubtitleCell(
                    text = stringResource(id = R.string.auto_connect_and_lockdown_mode_footer)
                )
            }

            item {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.auto_connect),
                    isToggled = uiState.isAutoConnectEnabled,
                    isEnabled = true,
                    onCellClicked = { newValue -> onToggleAutoConnect(newValue) }
                )
            }
            item {
                SwitchComposeSubtitleCell(text = stringResource(id = R.string.auto_connect_footer))
            }
            item {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.local_network_sharing),
                    isToggled = uiState.isAllowLanEnabled,
                    isEnabled = true,
                    onCellClicked = { newValue -> onToggleLocalNetworkSharing(newValue) },
                    onInfoClicked = { onLocalNetworkSharingInfoClick() }
                )
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
            }

            itemWithDivider {
                ExpandableComposeCell(
                    title = stringResource(R.string.dns_content_blockers_title),
                    isExpanded = expandContentBlockersState,
                    isEnabled = !uiState.isCustomDnsEnabled,
                    onInfoClicked = { onContentsBlockersInfoClick() },
                    onCellClicked = { expandContentBlockersState = !expandContentBlockersState }
                )
            }

            if (expandContentBlockersState) {
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_ads_title),
                        isToggled = uiState.contentBlockersOptions.blockAds,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAds(it) },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_trackers_title),
                        isToggled = uiState.contentBlockersOptions.blockTrackers,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockTrackers(it) },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_malware_title),
                        isToggled = uiState.contentBlockersOptions.blockMalware,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockMalware(it) },
                        onInfoClicked = { onMalwareInfoClick() },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_gambling_title),
                        isToggled = uiState.contentBlockersOptions.blockGambling,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockGambling(it) },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_adult_content_title),
                        isToggled = uiState.contentBlockersOptions.blockAdultContent,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAdultContent(it) },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }

                item {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_social_media_title),
                        isToggled = uiState.contentBlockersOptions.blockSocialMedia,
                        isEnabled = !uiState.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockSocialMedia(it) },
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = Dimens.indentedCellStartPadding
                    )
                }

                if (uiState.isCustomDnsEnabled) {
                    item {
                        ContentBlockersDisableModeCellSubtitle(
                            Modifier.background(MaterialTheme.colorScheme.secondary)
                                .padding(
                                    start = Dimens.cellStartPadding,
                                    top = topPadding,
                                    end = Dimens.cellEndPadding,
                                    bottom = Dimens.cellLabelVerticalPadding
                                )
                        )
                    }
                }
            }

            item {
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.enable_custom_dns),
                    isToggled = uiState.isCustomDnsEnabled,
                    isEnabled = uiState.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                    onInfoClicked = { onCustomDnsInfoClick() }
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
                        background = MaterialTheme.colorScheme.secondaryContainer,
                        startPadding = biggerPadding,
                    )
                }
            }

            item {
                CustomDnsCellSubtitle(
                    isCellClickable = uiState.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    modifier =
                        Modifier.background(MaterialTheme.colorScheme.secondary)
                            .padding(
                                start = Dimens.cellStartPadding,
                                top = topPadding,
                                end = Dimens.cellEndPadding,
                                bottom = Dimens.cellLabelVerticalPadding,
                            )
                )
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                InformationComposeCell(
                    title = stringResource(id = R.string.wireguard_port_title),
                    onInfoClicked = onWireguardPortInfoClicked
                )
            }

            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = uiState.selectedWireguardPort is Constraint.Any,
                    onCellClicked = { onWireguardPortSelected(Constraint.Any()) }
                )
            }

            WIREGUARD_PRESET_PORTS.forEach { port ->
                itemWithDivider {
                    SelectableCell(
                        title = port.toString(),
                        testTag = String.format(LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG, port),
                        isSelected = uiState.selectedWireguardPort.hasValue(port),
                        onCellClicked = { onWireguardPortSelected(Constraint.Only(Port(port))) }
                    )
                }
            }

            itemWithDivider {
                CustomPortCell(
                    title = stringResource(id = R.string.wireguard_custon_port_title),
                    isSelected = uiState.selectedWireguardPort.isCustom(),
                    port =
                        if (uiState.selectedWireguardPort.isCustom()) {
                            uiState.selectedWireguardPort.toDisplayCustomPort()
                        } else {
                            savedCustomPort.value.toDisplayCustomPort()
                        },
                    onMainCellClicked = {
                        if (savedCustomPort.value is Constraint.Only) {
                            onWireguardPortSelected(savedCustomPort.value)
                        } else {
                            onShowCustomPortDialog()
                        }
                    },
                    onPortCellClicked = { onShowCustomPortDialog() },
                    mainTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG,
                    numberTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
                )
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                InformationComposeCell(
                    title = stringResource(R.string.obfuscation_title),
                    onInfoClicked = { onObfuscationInfoClick() }
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = uiState.selectedObfuscation == SelectedObfuscation.Auto,
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Auto) }
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.obfuscation_on_udp_over_tcp),
                    isSelected = uiState.selectedObfuscation == SelectedObfuscation.Udp2Tcp,
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Udp2Tcp) }
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.off),
                    isSelected = uiState.selectedObfuscation == SelectedObfuscation.Off,
                    onCellClicked = { onSelectObfuscationSetting(SelectedObfuscation.Off) }
                )
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                InformationComposeCell(
                    title = stringResource(R.string.quantum_resistant_title),
                    onInfoClicked = { onQuantumResistanceInfoClicked() }
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = uiState.quantumResistant == QuantumResistantState.Auto,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.Auto) }
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.on),
                    testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG,
                    isSelected = uiState.quantumResistant == QuantumResistantState.On,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.On) }
                )
            }
            item {
                SelectableCell(
                    title = stringResource(id = R.string.off),
                    testTag = LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG,
                    isSelected = uiState.quantumResistant == QuantumResistantState.Off,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.Off) }
                )
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
            }

            item { MtuComposeCell(mtuValue = uiState.mtu, onEditMtu = { onMtuCellClick() }) }
            item {
                MtuSubtitle(modifier = Modifier.testTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
            }
        }
    }
}
