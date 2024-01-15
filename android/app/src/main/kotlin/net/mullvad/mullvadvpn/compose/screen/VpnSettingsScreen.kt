package net.mullvad.mullvadvpn.compose.screen

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
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
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
import net.mullvad.mullvadvpn.compose.destinations.AutoConnectAndLockdownModeScreenDestination
import net.mullvad.mullvadvpn.compose.destinations.ContentBlockersInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.CustomDnsInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.DnsDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.LocalNetworkSharingInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.MalwareInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.MtuDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.ObfuscationInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.QuantumResistanceInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.UdpOverTcpPortInfoDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.WireguardCustomPortDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.WireguardPortInfoDialogDestination
import net.mullvad.mullvadvpn.compose.dialog.WireguardCustomPortNavArgs
import net.mullvad.mullvadvpn.compose.dialog.WireguardPortInfoDialogArgument
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.common.util.vpnSettingsAvailable
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.Port
import net.mullvad.mullvadvpn.model.PortRange
import net.mullvad.mullvadvpn.model.QuantumResistantState
import net.mullvad.mullvadvpn.model.SelectedObfuscation
import net.mullvad.mullvadvpn.util.hasValue
import net.mullvad.mullvadvpn.util.isCustom
import net.mullvad.mullvadvpn.util.toValueOrNull
import net.mullvad.mullvadvpn.viewmodel.CustomDnsItem
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import org.koin.androidx.compose.koinViewModel

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
            snackbarHostState = SnackbarHostState(),
            onToggleBlockTrackers = {},
            onToggleBlockAds = {},
            onToggleBlockMalware = {},
            onToggleAutoConnect = {},
            onToggleLocalNetworkSharing = {},
            onToggleBlockAdultContent = {},
            onToggleBlockGambling = {},
            onToggleBlockSocialMedia = {},
            navigateToMtuDialog = {},
            navigateToDns = { _, _ -> },
            onToggleDnsClick = {},
            onBackClick = {},
            onSelectObfuscationSetting = {},
            onSelectQuantumResistanceSetting = {},
            onWireguardPortSelected = {},
        )
    }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun VpnSettings(
    navigator: DestinationsNavigator,
    dnsDialogResult: ResultRecipient<DnsDialogDestination, Boolean>,
    customWgPortResult: ResultRecipient<WireguardCustomPortDialogDestination, Int?>
) {
    val vm = koinViewModel<VpnSettingsViewModel>()
    val state = vm.uiState.collectAsState().value

    dnsDialogResult.onNavResult {
        when (it) {
            NavResult.Canceled -> {
                vm.onDnsDialogDismissed()
            }
            is NavResult.Value -> {}
        }
    }

    customWgPortResult.onNavResult {
        when (it) {
            NavResult.Canceled -> {}
            is NavResult.Value -> {
                val port = it.value

                if (port != null) {
                    vm.onWireguardPortSelected(Constraint.Only(Port(port)))
                } else {
                    vm.resetCustomPort()
                }
            }
        }
    }

    val snackbarHostState = remember { SnackbarHostState() }
    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect {
            when (it) {
                is VpnSettingsSideEffect.ShowToast ->
                    launch {
                        snackbarHostState.currentSnackbarData?.dismiss()
                        snackbarHostState.showSnackbar(message = it.message)
                    }
                VpnSettingsSideEffect.NavigateToDnsDialog ->
                    navigator.navigate(DnsDialogDestination(null, null)) { launchSingleTop = true }
            }
        }
    }

    val lifecycleOwner = LocalLifecycleOwner.current
    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            if (event == Lifecycle.Event.ON_STOP) {
                vm.onStopEvent()
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose { lifecycleOwner.lifecycle.removeObserver(observer) }
    }

    VpnSettingsScreen(
        uiState = state,
        snackbarHostState = snackbarHostState,
        navigateToContentBlockersInfo = {
            navigator.navigate(ContentBlockersInfoDialogDestination) { launchSingleTop = true }
        },
        navigateToAutoConnectScreen = {
            navigator.navigate(AutoConnectAndLockdownModeScreenDestination) {
                launchSingleTop = true
            }
        },
        navigateToCustomDnsInfo = {
            navigator.navigate(CustomDnsInfoDialogDestination) { launchSingleTop = true }
        },
        navigateToMalwareInfo = {
            navigator.navigate(MalwareInfoDialogDestination) { launchSingleTop = true }
        },
        navigateToObfuscationInfo = {
            navigator.navigate(ObfuscationInfoDialogDestination) { launchSingleTop = true }
        },
        navigateToQuantumResistanceInfo = {
            navigator.navigate(QuantumResistanceInfoDialogDestination) { launchSingleTop = true }
        },
        navigateUdp2TcpInfo = {
            navigator.navigate(UdpOverTcpPortInfoDialogDestination) { launchSingleTop = true }
        },
        navigateToWireguardPortInfo = {
            navigator.navigate(
                WireguardPortInfoDialogDestination(WireguardPortInfoDialogArgument(it))
            ) {
                launchSingleTop = true
            }
        },
        navigateToLocalNetworkSharingInfo = {
            navigator.navigate(LocalNetworkSharingInfoDialogDestination) { launchSingleTop = true }
        },
        onToggleBlockTrackers = vm::onToggleBlockTrackers,
        onToggleBlockAds = vm::onToggleBlockAds,
        onToggleBlockMalware = vm::onToggleBlockMalware,
        onToggleAutoConnect = vm::onToggleAutoConnect,
        onToggleLocalNetworkSharing = vm::onToggleLocalNetworkSharing,
        onToggleBlockAdultContent = vm::onToggleBlockAdultContent,
        onToggleBlockGambling = vm::onToggleBlockGambling,
        onToggleBlockSocialMedia = vm::onToggleBlockSocialMedia,
        navigateToMtuDialog = {
            navigator.navigate(MtuDialogDestination(it)) { launchSingleTop = true }
        },
        navigateToDns = { index, address ->
            navigator.navigate(DnsDialogDestination(index, address)) { launchSingleTop = true }
        },
        navigateToWireguardPortDialog = {
            val args =
                WireguardCustomPortNavArgs(
                    state.customWireguardPort?.toValueOrNull(),
                    state.availablePortRanges
                )
            navigator.navigate(WireguardCustomPortDialogDestination(args)) {
                launchSingleTop = true
            }
        },
        onToggleDnsClick = vm::onToggleCustomDns,
        onBackClick = navigator::navigateUp,
        onSelectObfuscationSetting = vm::onSelectObfuscationSetting,
        onSelectQuantumResistanceSetting = vm::onSelectQuantumResistanceSetting,
        onWireguardPortSelected = vm::onWireguardPortSelected,
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun VpnSettingsScreen(
    uiState: VpnSettingsUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    navigateToContentBlockersInfo: () -> Unit = {},
    navigateToAutoConnectScreen: () -> Unit = {},
    navigateToCustomDnsInfo: () -> Unit = {},
    navigateToMalwareInfo: () -> Unit = {},
    navigateToObfuscationInfo: () -> Unit = {},
    navigateToQuantumResistanceInfo: () -> Unit = {},
    navigateUdp2TcpInfo: () -> Unit = {},
    navigateToWireguardPortInfo: (availablePortRanges: List<PortRange>) -> Unit = {},
    navigateToLocalNetworkSharingInfo: () -> Unit = {},
    navigateToWireguardPortDialog: () -> Unit = {},
    onToggleBlockTrackers: (Boolean) -> Unit = {},
    onToggleBlockAds: (Boolean) -> Unit = {},
    onToggleBlockMalware: (Boolean) -> Unit = {},
    onToggleAutoConnect: (Boolean) -> Unit = {},
    onToggleLocalNetworkSharing: (Boolean) -> Unit = {},
    onToggleBlockAdultContent: (Boolean) -> Unit = {},
    onToggleBlockGambling: (Boolean) -> Unit = {},
    onToggleBlockSocialMedia: (Boolean) -> Unit = {},
    navigateToMtuDialog: (mtu: Int?) -> Unit = {},
    navigateToDns: (index: Int?, address: String?) -> Unit = { _, _ -> },
    onToggleDnsClick: (Boolean) -> Unit = {},
    onBackClick: () -> Unit = {},
    onSelectObfuscationSetting: (selectedObfuscation: SelectedObfuscation) -> Unit = {},
    onSelectQuantumResistanceSetting: (quantumResistant: QuantumResistantState) -> Unit = {},
    onWireguardPortSelected: (port: Constraint<Port>) -> Unit = {},
) {
    var expandContentBlockersState by rememberSaveable { mutableStateOf(false) }
    val biggerPadding = 54.dp
    val topPadding = 6.dp

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_vpn),
        navigationIcon = { NavigateBackIconButton(onBackClick) },
        snackbarHostState = snackbarHostState
    ) { modifier, lazyListState ->
        val context = LocalContext.current
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_TEST_TAG).animateContentSize(),
            state = lazyListState
        ) {
            if (context.vpnSettingsAvailable()) {
                item {
                    Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                    NavigationComposeCell(
                        title = stringResource(id = R.string.auto_connect_and_lockdown_mode),
                        onClick = { navigateToAutoConnectScreen() },
                    )
                }
                item {
                    SwitchComposeSubtitleCell(
                        text = stringResource(id = R.string.auto_connect_and_lockdown_mode_footer)
                    )
                }
            } else {
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
                    SwitchComposeSubtitleCell(
                        text = stringResource(id = R.string.auto_connect_footer)
                    )
                }
            }
            item {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.local_network_sharing),
                    isToggled = uiState.isLocalNetworkSharingEnabled,
                    isEnabled = true,
                    onCellClicked = { newValue -> onToggleLocalNetworkSharing(newValue) },
                    onInfoClicked = navigateToLocalNetworkSharingInfo
                )
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
            }

            itemWithDivider {
                ExpandableComposeCell(
                    title = stringResource(R.string.dns_content_blockers_title),
                    isExpanded = expandContentBlockersState,
                    isEnabled = !uiState.isCustomDnsEnabled,
                    onInfoClicked = { navigateToContentBlockersInfo() },
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
                        onInfoClicked = { navigateToMalwareInfo() },
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
                    onInfoClicked = { navigateToCustomDnsInfo() }
                )
            }

            if (uiState.isCustomDnsEnabled) {
                itemsIndexed(uiState.customDnsItems) { index, item ->
                    DnsCell(
                        address = item.address,
                        isUnreachableLocalDnsWarningVisible =
                            item.isLocal && !uiState.isLocalNetworkSharingEnabled,
                        onClick = { navigateToDns(index, item.address) },
                        modifier = Modifier.animateItemPlacement()
                    )
                    Divider()
                }

                itemWithDivider {
                    BaseCell(
                        onCellClicked = { navigateToDns(null, null) },
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
                    onInfoClicked = { navigateToWireguardPortInfo(uiState.availablePortRanges) },
                    onCellClicked = { navigateToWireguardPortInfo(uiState.availablePortRanges) },
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
                    port = uiState.customWireguardPort?.toValueOrNull(),
                    onMainCellClicked = {
                        if (uiState.customWireguardPort != null) {
                            onWireguardPortSelected(uiState.customWireguardPort)
                        } else {
                            navigateToWireguardPortDialog()
                        }
                    },
                    onPortCellClicked = navigateToWireguardPortDialog,
                    mainTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG,
                    numberTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
                )
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
                InformationComposeCell(
                    title = stringResource(R.string.obfuscation_title),
                    onInfoClicked = navigateToObfuscationInfo,
                    onCellClicked = navigateToObfuscationInfo
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
                    onInfoClicked = navigateToQuantumResistanceInfo,
                    onCellClicked = navigateToQuantumResistanceInfo
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

            item {
                MtuComposeCell(
                    mtuValue = uiState.mtu,
                    onEditMtu = { navigateToMtuDialog(uiState.mtu.toIntOrNull()) }
                )
            }
            item {
                MtuSubtitle(modifier = Modifier.testTag(LAZY_LIST_LAST_ITEM_TEST_TAG))
                Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding))
            }
        }
    }
}
