package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.dp
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.AutoConnectAndLockdownModeDestination
import com.ramcosta.composedestinations.generated.destinations.ContentBlockersInfoDestination
import com.ramcosta.composedestinations.generated.destinations.CustomDnsInfoDestination
import com.ramcosta.composedestinations.generated.destinations.DnsDestination
import com.ramcosta.composedestinations.generated.destinations.LocalNetworkSharingInfoDestination
import com.ramcosta.composedestinations.generated.destinations.MalwareInfoDestination
import com.ramcosta.composedestinations.generated.destinations.MtuDestination
import com.ramcosta.composedestinations.generated.destinations.ObfuscationInfoDestination
import com.ramcosta.composedestinations.generated.destinations.QuantumResistanceInfoDestination
import com.ramcosta.composedestinations.generated.destinations.ServerIpOverridesDestination
import com.ramcosta.composedestinations.generated.destinations.ShadowsocksSettingsDestination
import com.ramcosta.composedestinations.generated.destinations.Udp2TcpSettingsDestination
import com.ramcosta.composedestinations.generated.destinations.WireguardCustomPortDestination
import com.ramcosta.composedestinations.generated.destinations.WireguardPortInfoDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
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
import net.mullvad.mullvadvpn.compose.cell.ObfuscationModeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.communication.DnsDialogResult
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.dialog.CustomPortNavArgs
import net.mullvad.mullvadvpn.compose.dialog.info.WireguardPortInfoDialogArgument
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.preview.VpnSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.WIREGUARD_OBFUSCATION_OFF_CELL
import net.mullvad.mullvadvpn.compose.test.WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL
import net.mullvad.mullvadvpn.compose.test.WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.constant.WIREGUARD_PRESET_PORTS
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Default|NonDefault")
@Composable
private fun PreviewVpnSettings(
    @PreviewParameter(VpnSettingsUiStatePreviewParameterProvider::class) state: VpnSettingsUiState
) {
    AppTheme {
        VpnSettingsScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onToggleBlockTrackers = {},
            onToggleBlockAds = {},
            onToggleBlockMalware = {},
            onToggleLocalNetworkSharing = {},
            onToggleBlockAdultContent = {},
            onToggleBlockGambling = {},
            onToggleBlockSocialMedia = {},
            navigateToMtuDialog = {},
            navigateToDns = { _, _ -> },
            onToggleDnsClick = {},
            onBackClick = {},
            onSelectObfuscationMode = {},
            onSelectQuantumResistanceSetting = {},
            onWireguardPortSelected = {},
            navigateToShadowSocksSettings = {},
            navigateToUdp2TcpSettings = {},
            onToggleAutoStartAndConnectOnBoot = { _ -> },
            navigateToMalwareInfo = {},
            navigateToContentBlockersInfo = {},
            navigateToAutoConnectScreen = {},
            navigateToCustomDnsInfo = {},
            navigateToObfuscationInfo = {},
            navigateToQuantumResistanceInfo = {},
            navigateToWireguardPortInfo = {},
            navigateToLocalNetworkSharingInfo = {},
            navigateToWireguardPortDialog = {},
            navigateToServerIpOverrides = {},
            onSelectDeviceIpVersion = {},
            onToggleIPv6Toggle = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
@Suppress("LongMethod")
fun VpnSettings(
    navigator: DestinationsNavigator,
    dnsDialogResult: ResultRecipient<DnsDestination, DnsDialogResult>,
    customWgPortResult: ResultRecipient<WireguardCustomPortDestination, Port?>,
    mtuDialogResult: ResultRecipient<MtuDestination, Boolean>,
) {
    val vm = koinViewModel<VpnSettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    dnsDialogResult.OnNavResultValue { result ->
        when (result) {
            is DnsDialogResult.Success -> {
                vm.showApplySettingChangesWarningToast()
                if (result.isDnsListEmpty) {
                    vm.onToggleCustomDns(false)
                }
            }
            DnsDialogResult.Cancel -> vm.onDnsDialogDismissed()
            DnsDialogResult.Error -> {
                vm.showGenericErrorToast()
                vm.onDnsDialogDismissed()
            }
        }
    }

    customWgPortResult.OnNavResultValue { port ->
        if (port != null) {
            vm.onWireguardPortSelected(Constraint.Only(port))
        } else {
            vm.resetCustomPort()
        }
    }

    mtuDialogResult.OnNavResultValue { result ->
        if (!result) {
            vm.showGenericErrorToast()
        }
    }

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            is VpnSettingsSideEffect.ShowToast ->
                launch { snackbarHostState.showSnackbarImmediately(message = it.message(context)) }
            VpnSettingsSideEffect.NavigateToDnsDialog ->
                navigator.navigate(DnsDestination(null, null)) { launchSingleTop = true }
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
        state = state,
        snackbarHostState = snackbarHostState,
        navigateToContentBlockersInfo =
            dropUnlessResumed { navigator.navigate(ContentBlockersInfoDestination) },
        navigateToAutoConnectScreen =
            dropUnlessResumed { navigator.navigate(AutoConnectAndLockdownModeDestination) },
        navigateToCustomDnsInfo =
            dropUnlessResumed { navigator.navigate(CustomDnsInfoDestination) },
        navigateToMalwareInfo = dropUnlessResumed { navigator.navigate(MalwareInfoDestination) },
        navigateToObfuscationInfo =
            dropUnlessResumed { navigator.navigate(ObfuscationInfoDestination) },
        navigateToQuantumResistanceInfo =
            dropUnlessResumed { navigator.navigate(QuantumResistanceInfoDestination) },
        navigateToWireguardPortInfo =
            dropUnlessResumed { availablePortRanges: List<PortRange> ->
                navigator.navigate(
                    WireguardPortInfoDestination(
                        WireguardPortInfoDialogArgument(availablePortRanges)
                    )
                )
            },
        navigateToLocalNetworkSharingInfo =
            dropUnlessResumed { navigator.navigate(LocalNetworkSharingInfoDestination) },
        navigateToServerIpOverrides =
            dropUnlessResumed { navigator.navigate(ServerIpOverridesDestination) },
        onToggleBlockTrackers = vm::onToggleBlockTrackers,
        onToggleBlockAds = vm::onToggleBlockAds,
        onToggleBlockMalware = vm::onToggleBlockMalware,
        onToggleLocalNetworkSharing = vm::onToggleLocalNetworkSharing,
        onToggleBlockAdultContent = vm::onToggleBlockAdultContent,
        onToggleBlockGambling = vm::onToggleBlockGambling,
        onToggleBlockSocialMedia = vm::onToggleBlockSocialMedia,
        navigateToMtuDialog =
            dropUnlessResumed { mtu: Mtu? -> navigator.navigate(MtuDestination(mtu)) },
        navigateToDns =
            dropUnlessResumed { index: Int?, address: String? ->
                navigator.navigate(DnsDestination(index, address))
            },
        navigateToWireguardPortDialog =
            dropUnlessResumed {
                navigator.navigate(
                    WireguardCustomPortDestination(
                        CustomPortNavArgs(
                            customPort = state.customWireguardPort,
                            allowedPortRanges = state.availablePortRanges,
                        )
                    )
                )
            },
        onToggleDnsClick = vm::onToggleCustomDns,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onSelectObfuscationMode = vm::onSelectObfuscationMode,
        onSelectQuantumResistanceSetting = vm::onSelectQuantumResistanceSetting,
        onWireguardPortSelected = vm::onWireguardPortSelected,
        navigateToShadowSocksSettings =
            dropUnlessResumed { navigator.navigate(ShadowsocksSettingsDestination) },
        navigateToUdp2TcpSettings =
            dropUnlessResumed { navigator.navigate(Udp2TcpSettingsDestination) },
        onToggleAutoStartAndConnectOnBoot = vm::onToggleAutoStartAndConnectOnBoot,
        onSelectDeviceIpVersion = vm::onDeviceIpVersionSelected,
        onToggleIPv6Toggle = vm::setIpV6Enabled,
    )
}

@Suppress("LongMethod", "LongParameterList")
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun VpnSettingsScreen(
    state: VpnSettingsUiState,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    navigateToContentBlockersInfo: () -> Unit,
    navigateToAutoConnectScreen: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToObfuscationInfo: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToWireguardPortInfo: (availablePortRanges: List<PortRange>) -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToWireguardPortDialog: () -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleLocalNetworkSharing: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    navigateToMtuDialog: (mtu: Mtu?) -> Unit,
    navigateToDns: (index: Int?, address: String?) -> Unit,
    onToggleDnsClick: (Boolean) -> Unit,
    onBackClick: () -> Unit,
    onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit,
    onSelectQuantumResistanceSetting: (quantumResistant: QuantumResistantState) -> Unit,
    onWireguardPortSelected: (port: Constraint<Port>) -> Unit,
    navigateToShadowSocksSettings: () -> Unit,
    navigateToUdp2TcpSettings: () -> Unit,
    onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit,
    onSelectDeviceIpVersion: (ipVersion: Constraint<IpVersion>) -> Unit,
    onToggleIPv6Toggle: (Boolean) -> Unit,
) {
    var expandContentBlockersState by rememberSaveable { mutableStateOf(false) }
    val biggerPadding = 54.dp
    val topPadding = 6.dp

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings_vpn),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        snackbarHostState = snackbarHostState,
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG).animateContentSize(),
            state = lazyListState,
        ) {
            if (state.systemVpnSettingsAvailable) {
                item {
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
                    HeaderSwitchComposeCell(
                        title = stringResource(R.string.connect_on_start),
                        isToggled = state.autoStartAndConnectOnBoot,
                        onCellClicked = { newValue -> onToggleAutoStartAndConnectOnBoot(newValue) },
                    )
                    SwitchComposeSubtitleCell(
                        text =
                            textResource(
                                R.string.connect_on_start_footer,
                                textResource(R.string.auto_connect_and_lockdown_mode),
                            )
                    )
                }
            }

            item {
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.local_network_sharing),
                    isToggled = state.isLocalNetworkSharingEnabled,
                    isEnabled = true,
                    onCellClicked = { newValue -> onToggleLocalNetworkSharing(newValue) },
                    onInfoClicked = navigateToLocalNetworkSharingInfo,
                )
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
            }

            itemWithDivider {
                ExpandableComposeCell(
                    title = stringResource(R.string.dns_content_blockers_title),
                    isExpanded = expandContentBlockersState,
                    isEnabled = !state.isCustomDnsEnabled,
                    onInfoClicked = { navigateToContentBlockersInfo() },
                    onCellClicked = { expandContentBlockersState = !expandContentBlockersState },
                )
            }

            if (expandContentBlockersState) {
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_ads_title),
                        isToggled = state.contentBlockersOptions.blockAds,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAds(it) },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_trackers_title),
                        isToggled = state.contentBlockersOptions.blockTrackers,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockTrackers(it) },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_malware_title),
                        isToggled = state.contentBlockersOptions.blockMalware,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockMalware(it) },
                        onInfoClicked = { navigateToMalwareInfo() },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_gambling_title),
                        isToggled = state.contentBlockersOptions.blockGambling,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockGambling(it) },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }
                itemWithDivider {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_adult_content_title),
                        isToggled = state.contentBlockersOptions.blockAdultContent,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockAdultContent(it) },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }

                item {
                    NormalSwitchComposeCell(
                        title = stringResource(R.string.block_social_media_title),
                        isToggled = state.contentBlockersOptions.blockSocialMedia,
                        isEnabled = !state.isCustomDnsEnabled,
                        onCellClicked = { onToggleBlockSocialMedia(it) },
                        background = MaterialTheme.colorScheme.surfaceContainerLow,
                        startPadding = Dimens.indentedCellStartPadding,
                    )
                }

                if (state.isCustomDnsEnabled) {
                    item {
                        ContentBlockersDisableModeCellSubtitle(
                            Modifier.background(MaterialTheme.colorScheme.surface)
                                .padding(
                                    start = Dimens.cellStartPadding,
                                    top = topPadding,
                                    end = Dimens.cellEndPadding,
                                    bottom = Dimens.cellVerticalSpacing,
                                )
                        )
                    }
                }
            }

            item {
                HeaderSwitchComposeCell(
                    title = stringResource(R.string.enable_custom_dns),
                    isToggled = state.isCustomDnsEnabled,
                    isEnabled = state.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                    onInfoClicked = { navigateToCustomDnsInfo() },
                )
            }

            if (state.isCustomDnsEnabled) {
                itemsIndexedWithDivider(state.customDnsItems) { index, item ->
                    DnsCell(
                        address = item.address,
                        isUnreachableLocalDnsWarningVisible =
                            item.isLocal && !state.isLocalNetworkSharingEnabled,
                        isUnreachableIpv6DnsWarningVisible =
                            item.isIpv6 && !state.isIPv6Enabled,
                        onClick = { navigateToDns(index, item.address) },
                        modifier = Modifier.animateItem(),
                    )
                }

                if (state.customDnsItems.isNotEmpty()) {
                    itemWithDivider {
                        BaseCell(
                            onCellClicked = { navigateToDns(null, null) },
                            headlineContent = {
                                Text(
                                    text = stringResource(id = R.string.add_a_server),
                                    color = MaterialTheme.colorScheme.onSurface,
                                )
                            },
                            bodyView = {},
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = biggerPadding,
                        )
                    }
                }
            }

            item {
                CustomDnsCellSubtitle(
                    isCellClickable = state.contentBlockersOptions.isAnyBlockerEnabled().not(),
                    modifier =
                        Modifier.padding(
                            start = Dimens.cellStartPadding,
                            top = topPadding,
                            end = Dimens.cellEndPadding,
                            bottom = Dimens.cellVerticalSpacing,
                        ),
                )
            }

            itemWithDivider {
                InformationComposeCell(
                    title = stringResource(id = R.string.wireguard_port_title),
                    onInfoClicked = { navigateToWireguardPortInfo(state.availablePortRanges) },
                    onCellClicked = { navigateToWireguardPortInfo(state.availablePortRanges) },
                    isEnabled = state.isWireguardPortEnabled,
                )
            }

            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.selectedWireguardPort == Constraint.Any,
                    onCellClicked = { onWireguardPortSelected(Constraint.Any) },
                    isEnabled = state.isWireguardPortEnabled,
                )
            }

            WIREGUARD_PRESET_PORTS.forEach { port ->
                itemWithDivider {
                    SelectableCell(
                        title = port.toString(),
                        testTag =
                            String.format(
                                null,
                                LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG,
                                port.value,
                            ),
                        isSelected = state.selectedWireguardPort.getOrNull() == port,
                        onCellClicked = { onWireguardPortSelected(Constraint.Only(port)) },
                        isEnabled = state.isWireguardPortEnabled,
                    )
                }
            }

            itemWithDivider {
                CustomPortCell(
                    title = stringResource(id = R.string.wireguard_custon_port_title),
                    isSelected = state.isCustomWireguardPort,
                    port = state.customWireguardPort,
                    onMainCellClicked = {
                        if (state.customWireguardPort != null) {
                            onWireguardPortSelected(Constraint.Only(state.customWireguardPort))
                        } else {
                            navigateToWireguardPortDialog()
                        }
                    },
                    onPortCellClicked = navigateToWireguardPortDialog,
                    isEnabled = state.isWireguardPortEnabled,
                    mainTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG,
                    numberTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG,
                )
            }

            if (!state.isWireguardPortEnabled) {
                item {
                    Text(
                        text =
                            stringResource(
                                id = R.string.wg_port_subtitle,
                                stringResource(R.string.wireguard),
                            ),
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier =
                            Modifier.padding(
                                start = Dimens.cellStartPadding,
                                top = topPadding,
                                end = Dimens.cellEndPadding,
                            ),
                    )
                }
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
                InformationComposeCell(
                    title = stringResource(R.string.obfuscation_title),
                    onInfoClicked = navigateToObfuscationInfo,
                    onCellClicked = navigateToObfuscationInfo,
                    testTag = LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG,
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.obfuscationMode == ObfuscationMode.Auto,
                    onCellClicked = { onSelectObfuscationMode(ObfuscationMode.Auto) },
                )
            }
            itemWithDivider {
                ObfuscationModeCell(
                    obfuscationMode = ObfuscationMode.Shadowsocks,
                    isSelected = state.obfuscationMode == ObfuscationMode.Shadowsocks,
                    port = state.selectedShadowsSocksObfuscationPort,
                    onSelected = onSelectObfuscationMode,
                    onNavigate = navigateToShadowSocksSettings,
                    testTag = WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL,
                )
            }
            itemWithDivider {
                ObfuscationModeCell(
                    obfuscationMode = ObfuscationMode.Udp2Tcp,
                    isSelected = state.obfuscationMode == ObfuscationMode.Udp2Tcp,
                    port = state.selectedUdp2TcpObfuscationPort,
                    onSelected = onSelectObfuscationMode,
                    onNavigate = navigateToUdp2TcpSettings,
                    testTag = WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL,
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.off),
                    isSelected = state.obfuscationMode == ObfuscationMode.Off,
                    onCellClicked = { onSelectObfuscationMode(ObfuscationMode.Off) },
                    testTag = WIREGUARD_OBFUSCATION_OFF_CELL,
                )
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
                InformationComposeCell(
                    title = stringResource(R.string.quantum_resistant_title),
                    onInfoClicked = navigateToQuantumResistanceInfo,
                    onCellClicked = navigateToQuantumResistanceInfo,
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.quantumResistant == QuantumResistantState.Auto,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.Auto) },
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.on),
                    testTag = LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG,
                    isSelected = state.quantumResistant == QuantumResistantState.On,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.On) },
                )
            }
            item {
                SelectableCell(
                    title = stringResource(id = R.string.off),
                    testTag = LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG,
                    isSelected = state.quantumResistant == QuantumResistantState.Off,
                    onCellClicked = { onSelectQuantumResistanceSetting(QuantumResistantState.Off) },
                )
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
            }

            itemWithDivider {
                InformationComposeCell(title = stringResource(R.string.device_ip_version_title))
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.automatic),
                    isSelected = state.deviceIpVersion == Constraint.Any,
                    onCellClicked = { onSelectDeviceIpVersion(Constraint.Any) },
                )
            }
            itemWithDivider {
                SelectableCell(
                    title = stringResource(id = R.string.device_ip_version_ipv4),
                    isSelected = state.deviceIpVersion.getOrNull() == IpVersion.IPV4,
                    onCellClicked = { onSelectDeviceIpVersion(Constraint.Only(IpVersion.IPV4)) },
                )
            }
            item {
                SelectableCell(
                    title = stringResource(id = R.string.device_ip_version_ipv6),
                    isSelected = state.deviceIpVersion.getOrNull() == IpVersion.IPV6,
                    onCellClicked = { onSelectDeviceIpVersion(Constraint.Only(IpVersion.IPV6)) },
                )
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
            }

            item {
                MtuComposeCell(mtuValue = state.mtu, onEditMtu = { navigateToMtuDialog(state.mtu) })
            }
            item { MtuSubtitle(modifier = Modifier.testTag(LAZY_LIST_LAST_ITEM_TEST_TAG)) }

            item {
                HeaderSwitchComposeCell(
                    title = "Enable IPv6",
                    isToggled = state.isIPv6Enabled,
                    isEnabled = true,
                    onCellClicked = { newValue -> onToggleIPv6Toggle(newValue) },
                )
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
            }

            item { ServerIpOverrides(navigateToServerIpOverrides) }
        }
    }
}

@Composable
private fun ServerIpOverrides(onServerIpOverridesClick: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(id = R.string.server_ip_overrides),
        onClick = onServerIpOverridesClick,
    )
}

private fun VpnSettingsSideEffect.ShowToast.message(context: Context) =
    when (this) {
        VpnSettingsSideEffect.ShowToast.ApplySettingsWarning ->
            context.getString(R.string.settings_changes_effect_warning_short)
        VpnSettingsSideEffect.ShowToast.GenericError -> context.getString(R.string.error_occurred)
    }
