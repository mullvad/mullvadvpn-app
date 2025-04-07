@file:OptIn(ExperimentalSharedTransitionApi::class, ExperimentalMaterial3Api::class)

package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import android.os.Parcelable
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MutableState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.AutoConnectAndLockdownModeDestination
import com.ramcosta.composedestinations.generated.destinations.ContentBlockersInfoDestination
import com.ramcosta.composedestinations.generated.destinations.CustomDnsInfoDestination
import com.ramcosta.composedestinations.generated.destinations.DnsDestination
import com.ramcosta.composedestinations.generated.destinations.Ipv6InfoDestination
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
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.BaseSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.ContentBlockersDisableModeCellSubtitle
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
import net.mullvad.mullvadvpn.compose.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.dialog.CustomPortNavArgs
import net.mullvad.mullvadvpn.compose.dialog.info.WireguardPortInfoDialogArgument
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.VpnSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
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
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.PortRange
import net.mullvad.mullvadvpn.lib.model.QuantumResistantState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.util.indexOfFirstOrNull
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsUiState
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
            initialScrollToFeature = null,
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
            navigateToWireguardPortDialog = { _, _ -> },
            navigateToServerIpOverrides = {},
            onSelectDeviceIpVersion = {},
            onToggleIpv6 = {},
            onToggleContentBlockersExpanded = {},
            navigateToIpv6Info = {},
        )
    }
}

@Parcelize
data class VpnSettingsNavArgs(
    val scrollToFeature: FeatureIndicator? = null,
    val isModal: Boolean = false,
) : Parcelable

@Destination<RootGraph>(
    style = SlideInFromRightTransition::class,
    navArgs = VpnSettingsNavArgs::class,
)
@Composable
@Suppress("LongMethod")
fun SharedTransitionScope.VpnSettings(
    navigator: DestinationsNavigator,
    animatedVisibilityScope: AnimatedVisibilityScope,
    navArgs: VpnSettingsNavArgs,
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
            }
            DnsDialogResult.Error -> {
                vm.showGenericErrorToast()
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

    VpnSettingsScreen(
        state = state,
        initialScrollToFeature = navArgs.scrollToFeature,
        modifier =
            Modifier.sharedBounds(
                rememberSharedContentState(key = navArgs.scrollToFeature ?: ""),
                animatedVisibilityScope = animatedVisibilityScope,
            ),
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
            dropUnlessResumed { navigator.navigate(ServerIpOverridesDestination()) },
        onToggleContentBlockersExpanded = vm::onToggleContentBlockersExpand,
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
            dropUnlessResumed { customPort, availablePortRanges ->
                navigator.navigate(
                    WireguardCustomPortDestination(
                        CustomPortNavArgs(
                            customPort = customPort,
                            allowedPortRanges = availablePortRanges,
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
        onToggleIpv6 = vm::setIpv6Enabled,
        navigateToIpv6Info = dropUnlessResumed { navigator.navigate(Ipv6InfoDestination) },
    )
}

@Suppress("LongParameterList")
@Composable
fun VpnSettingsScreen(
    state: VpnSettingsUiState,
    initialScrollToFeature: FeatureIndicator?,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    navigateToContentBlockersInfo: () -> Unit,
    navigateToAutoConnectScreen: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToObfuscationInfo: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToWireguardPortInfo: (availablePortRanges: List<PortRange>) -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToWireguardPortDialog:
        (customPort: Port?, availablePortRanges: List<PortRange>) -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    onToggleContentBlockersExpanded: () -> Unit,
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
    onToggleIpv6: (Boolean) -> Unit,
    navigateToIpv6Info: () -> Unit,
) {
    val appBarState = rememberTopAppBarState()
    val canScroll = remember { mutableStateOf(false) }
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(
            appBarState,
            canScroll = { canScroll.value },
        )
    Scaffold(
        modifier = modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MullvadMediumTopBar(
                title = stringResource(id = R.string.settings_vpn),
                navigationIcon = {
                    if (state.isModal) {
                        NavigateCloseIconButton(onNavigateClose = onBackClick)
                    } else {
                        NavigateBackIconButton(onNavigateBack = onBackClick)
                    }
                },
                scrollBehavior = scrollBehavior,
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        content = {
            when (state) {
                is VpnSettingsUiState.Loading ->
                    Row(
                        modifier.fillMaxWidth().padding(it),
                        horizontalArrangement = Arrangement.Center,
                    ) {
                        CircularProgressIndicator()
                    }

                is VpnSettingsUiState.Content ->
                    VpnSettingsContent(
                        modifier.padding(it),
                        state,
                        initialScrollToFeature,
                        canScroll,
                        navigateToContentBlockersInfo,
                        navigateToAutoConnectScreen,
                        navigateToCustomDnsInfo,
                        navigateToMalwareInfo,
                        navigateToObfuscationInfo,
                        navigateToQuantumResistanceInfo,
                        navigateToWireguardPortInfo,
                        navigateToLocalNetworkSharingInfo,
                        navigateToWireguardPortDialog,
                        navigateToServerIpOverrides,
                        onToggleContentBlockersExpanded,
                        onToggleBlockTrackers,
                        onToggleBlockAds,
                        onToggleBlockMalware,
                        onToggleLocalNetworkSharing,
                        onToggleBlockAdultContent,
                        onToggleBlockGambling,
                        onToggleBlockSocialMedia,
                        navigateToMtuDialog,
                        navigateToDns,
                        onToggleDnsClick,
                        onSelectObfuscationMode,
                        onSelectQuantumResistanceSetting,
                        onWireguardPortSelected,
                        navigateToShadowSocksSettings,
                        navigateToUdp2TcpSettings,
                        onToggleAutoStartAndConnectOnBoot,
                        onSelectDeviceIpVersion,
                        onToggleIpv6,
                        navigateToIpv6Info,
                    )
            }
        },
    )
}

@Suppress("LongMethod", "LongParameterList", "CyclomaticComplexMethod")
@Composable
fun VpnSettingsContent(
    modifier: Modifier = Modifier,
    state: VpnSettingsUiState.Content,
    initialScrollToFeature: FeatureIndicator?,
    canScroll: MutableState<Boolean>,
    navigateToContentBlockersInfo: () -> Unit,
    navigateToAutoConnectScreen: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToObfuscationInfo: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToWireguardPortInfo: (availablePortRanges: List<PortRange>) -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToWireguardPortDialog:
        (customPort: Port?, availablePortRanges: List<PortRange>) -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    onToggleContentBlockersExpanded: () -> Unit,
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
    onSelectObfuscationMode: (obfuscationMode: ObfuscationMode) -> Unit,
    onSelectQuantumResistanceSetting: (quantumResistant: QuantumResistantState) -> Unit,
    onWireguardPortSelected: (port: Constraint<Port>) -> Unit,
    navigateToShadowSocksSettings: () -> Unit,
    navigateToUdp2TcpSettings: () -> Unit,
    onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit,
    onSelectDeviceIpVersion: (ipVersion: Constraint<IpVersion>) -> Unit,
    onToggleIpv6: (Boolean) -> Unit,
    navigateToIpv6Info: () -> Unit,
) {
    val initialIndexFocus =
        when (initialScrollToFeature) {
            FeatureIndicator.UDP_2_TCP,
            FeatureIndicator.SHADOWSOCKS -> VpnSettingItem.ObfuscationHeader::class
            FeatureIndicator.LAN_SHARING -> VpnSettingItem.LocalNetworkSharingHeader::class
            FeatureIndicator.QUANTUM_RESISTANCE -> VpnSettingItem.QuantumResistanceHeader::class
            FeatureIndicator.DNS_CONTENT_BLOCKERS -> VpnSettingItem.DnsContentBlockers::class
            FeatureIndicator.CUSTOM_MTU -> VpnSettingItem.MtuHeader::class
            else -> null
        }?.let { clazz -> state.settings.indexOfFirstOrNull { it::class == clazz } ?: 0 } ?: 0

    val highlightAnimation = remember { Animatable(1f) }
    if (initialScrollToFeature != null) {
        LaunchedEffect(Unit) {
            repeat(times = 3) {
                highlightAnimation.animateTo(0f)
                highlightAnimation.animateTo(1f)
            }
        }
    }

    val lazyListState = rememberLazyListState(initialIndexFocus)
    canScroll.value = lazyListState.canScrollForward || lazyListState.canScrollBackward
    LazyColumn(
        modifier =
            modifier
                .testTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .fillMaxSize()
                .drawVerticalScrollbar(
                    state = lazyListState,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .animateContentSize(),
        state = lazyListState,
    ) {
        state.settings.forEach {
            when (it) {
                VpnSettingItem.AutoConnectAndLockdownModeHeader ->
                    item(key = it.javaClass.simpleName) {
                        NavigationComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(id = R.string.auto_connect_and_lockdown_mode),
                            onClick = { navigateToAutoConnectScreen() },
                        )
                    }

                VpnSettingItem.AutoConnectAndLockdownModeInfo ->
                    item(key = it.javaClass.simpleName) {
                        SwitchComposeSubtitleCell(
                            modifier = Modifier.animateItem(),
                            text =
                                stringResource(id = R.string.auto_connect_and_lockdown_mode_footer),
                        )
                    }

                is VpnSettingItem.ConnectDeviceOnStartUpHeader ->
                    item(key = it.javaClass.simpleName) {
                        HeaderSwitchComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.connect_on_start),
                            isToggled = it.enabled,
                            onCellClicked = { newValue ->
                                onToggleAutoStartAndConnectOnBoot(newValue)
                            },
                        )
                    }

                VpnSettingItem.ConnectDeviceOnStartUpInfo ->
                    item(key = it.javaClass.simpleName) {
                        SwitchComposeSubtitleCell(
                            modifier = Modifier.animateItem(),
                            text =
                                textResource(
                                    R.string.connect_on_start_footer,
                                    textResource(R.string.auto_connect_and_lockdown_mode),
                                ),
                        )
                    }

                VpnSettingItem.CustomDnsAdd ->
                    item(key = it.javaClass.simpleName) {
                        BaseCell(
                            modifier = Modifier.animateItem(),
                            onCellClicked = { navigateToDns(null, null) },
                            headlineContent = {
                                Text(
                                    text = stringResource(id = R.string.add_a_server),
                                    color = MaterialTheme.colorScheme.onSurface,
                                )
                            },
                            bodyView = {},
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.cellStartPaddingLarge,
                        )
                    }

                is VpnSettingItem.CustomDnsEntry ->
                    item(key = it.javaClass.simpleName + it.index) {
                        DnsCell(
                            address = it.customDnsItem.address,
                            isUnreachableLocalDnsWarningVisible = it.showUnreachableLocalDnsWarning,
                            isUnreachableIpv6DnsWarningVisible = it.showUnreachableIpv6DnsWarning,
                            onClick = { navigateToDns(it.index, it.customDnsItem.address) },
                            modifier = Modifier.animateItem(),
                        )
                    }

                VpnSettingItem.CustomDnsInfo ->
                    item(key = it.javaClass.simpleName) {
                        BaseSubtitleCell(
                            text = textResource(id = R.string.custom_dns_footer),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.CustomDnsServerHeader ->
                    item(key = it.javaClass.simpleName) {
                        HeaderSwitchComposeCell(
                            title = stringResource(R.string.enable_custom_dns),
                            isToggled = it.enabled,
                            isEnabled = it.isOptionEnabled,
                            onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                            onInfoClicked = { navigateToCustomDnsInfo() },
                            background =
                                if (initialScrollToFeature == FeatureIndicator.CUSTOM_DNS) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                            modifier = Modifier.animateItem(),
                        )
                    }
                VpnSettingItem.CustomDnsUnavailable ->
                    item(key = it.javaClass.simpleName) {
                        BaseSubtitleCell(
                            textResource(
                                id = R.string.custom_dns_disable_mode_subtitle,
                                textResource(id = R.string.dns_content_blockers),
                            ),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.animateItem(),
                        )
                    }

                VpnSettingItem.DeviceIpVersionHeader ->
                    item(key = it.javaClass.simpleName) {
                        InformationComposeCell(
                            title = stringResource(R.string.device_ip_version_title),
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.DeviceIpVersionItem ->
                    item(key = it.javaClass.simpleName + it.constraint.getOrNull().toString()) {
                        SelectableCell(
                            modifier = Modifier.animateItem(),
                            title =
                                when (it.constraint) {
                                    Constraint.Any -> stringResource(id = R.string.automatic)

                                    is Constraint.Only ->
                                        when (it.constraint.value) {
                                            IpVersion.IPV4 -> stringResource(id = R.string.ipv4)

                                            IpVersion.IPV6 -> stringResource(id = R.string.ipv6)
                                        }
                                },
                            isSelected = it.selected,
                            onCellClicked = { onSelectDeviceIpVersion(it.constraint) },
                        )
                    }

                VpnSettingItem.DeviceIpVersionInfo -> {
                    item(key = it.javaClass.simpleName) {
                        BaseSubtitleCell(
                            text = stringResource(R.string.device_ip_version_subtitle),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.animateItem(),
                        )
                    }
                }

                VpnSettingItem.Divider -> {
                    item(contentType = it.javaClass.simpleName) {
                        HorizontalDivider(
                            modifier = Modifier.animateItem(),
                            color = Color.Transparent,
                        )
                    }
                }

                is VpnSettingItem.DnsContentBlockerItem.Ads ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            title = stringResource(R.string.block_ads_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockAds(it) },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.AdultContent ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            title = stringResource(R.string.block_adult_content_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockAdultContent(it) },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.Gambling ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            title = stringResource(R.string.block_gambling_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockGambling(it) },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.Malware ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.block_malware_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockMalware(it) },
                            onInfoClicked = { navigateToMalwareInfo() },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.SocialMedia ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.block_social_media_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockSocialMedia(it) },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.Trackers ->
                    item(key = it.javaClass.simpleName) {
                        NormalSwitchComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.block_trackers_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockTrackers(it) },
                            background = MaterialTheme.colorScheme.surfaceContainerLow,
                            startPadding = Dimens.indentedCellStartPadding,
                        )
                    }

                is VpnSettingItem.DnsContentBlockers ->
                    item(key = it.javaClass.simpleName) {
                        ExpandableComposeCell(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.dns_content_blockers),
                            background =
                                if (
                                    initialScrollToFeature == FeatureIndicator.DNS_CONTENT_BLOCKERS
                                ) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                            isExpanded = it.expanded,
                            isEnabled = it.featureEnabled,
                            onInfoClicked = { navigateToContentBlockersInfo() },
                            onCellClicked = { onToggleContentBlockersExpanded() },
                        )
                    }

                VpnSettingItem.DnsContentBlockersUnavailable ->
                    item(key = it.javaClass.simpleName) {
                        ContentBlockersDisableModeCellSubtitle(modifier = Modifier.animateItem())
                    }

                is VpnSettingItem.EnableIpv6Header ->
                    item(key = it.javaClass.simpleName) {
                        HeaderSwitchComposeCell(
                            title = stringResource(R.string.enable_ipv6),
                            isToggled = it.enabled,
                            isEnabled = true,
                            onCellClicked = onToggleIpv6,
                            onInfoClicked = navigateToIpv6Info,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.LocalNetworkSharingHeader ->
                    item(key = it.javaClass.simpleName) {
                        HeaderSwitchComposeCell(
                            background =
                                if (initialScrollToFeature == FeatureIndicator.LAN_SHARING) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                            title = stringResource(R.string.local_network_sharing),
                            isToggled = it.enabled,
                            isEnabled = true,
                            modifier = Modifier.animateItem(),
                            onCellClicked = { newValue -> onToggleLocalNetworkSharing(newValue) },
                            onInfoClicked = navigateToLocalNetworkSharingInfo,
                        )
                    }

                is VpnSettingItem.MtuHeader ->
                    item(key = it.javaClass.simpleName) {
                        MtuComposeCell(
                            mtuValue = it.mtu,
                            onEditMtu = { navigateToMtuDialog(it.mtu) },
                            modifier = Modifier.animateItem(),
                            background =
                                if (initialScrollToFeature == FeatureIndicator.CUSTOM_MTU) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                        )
                    }

                VpnSettingItem.MtuInfo ->
                    item(key = it.javaClass.simpleName) {
                        MtuSubtitle(
                            modifier = Modifier.testTag(LAZY_LIST_LAST_ITEM_TEST_TAG).animateItem()
                        )
                    }

                VpnSettingItem.ObfuscationHeader ->
                    item(key = it.javaClass.simpleName) {
                        InformationComposeCell(
                            title = stringResource(R.string.obfuscation_title),
                            onInfoClicked = navigateToObfuscationInfo,
                            onCellClicked = navigateToObfuscationInfo,
                            background =
                                if (
                                    initialScrollToFeature == FeatureIndicator.UDP_2_TCP ||
                                        initialScrollToFeature == FeatureIndicator.SHADOWSOCKS
                                ) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                            testTag = LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.ObfuscationItem.Automatic ->
                    item(key = it.javaClass.simpleName) {
                        SelectableCell(
                            title = stringResource(id = R.string.automatic),
                            isSelected = it.selected,
                            onCellClicked = { onSelectObfuscationMode(ObfuscationMode.Auto) },
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.ObfuscationItem.Off ->
                    item(key = it.javaClass.simpleName) {
                        SelectableCell(
                            title = stringResource(id = R.string.off),
                            isSelected = it.selected,
                            onCellClicked = { onSelectObfuscationMode(ObfuscationMode.Off) },
                            testTag = WIREGUARD_OBFUSCATION_OFF_CELL,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.ObfuscationItem.Shadowsocks ->
                    item(key = it.javaClass.simpleName) {
                        ObfuscationModeCell(
                            obfuscationMode = ObfuscationMode.Shadowsocks,
                            isSelected = it.selected,
                            port = it.port,
                            onSelected = onSelectObfuscationMode,
                            onNavigate = navigateToShadowSocksSettings,
                            testTag = WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.ObfuscationItem.UdpOverTcp ->
                    item(key = it.javaClass.simpleName) {
                        ObfuscationModeCell(
                            obfuscationMode = ObfuscationMode.Udp2Tcp,
                            isSelected = it.selected,
                            port = it.port,
                            onSelected = onSelectObfuscationMode,
                            onNavigate = navigateToUdp2TcpSettings,
                            testTag = WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.QuantumItem ->
                    item(key = it.javaClass.simpleName + it.quantumResistantState) {
                        SelectableCell(
                            title =
                                when (it.quantumResistantState) {
                                    QuantumResistantState.Auto ->
                                        stringResource(id = R.string.automatic)

                                    QuantumResistantState.Off -> stringResource(id = R.string.off)

                                    QuantumResistantState.On -> stringResource(id = R.string.on)
                                },
                            isSelected = it.selected,
                            modifier = Modifier.animateItem(),
                            onCellClicked = {
                                onSelectQuantumResistanceSetting(it.quantumResistantState)
                            },
                            testTag =
                                when (it.quantumResistantState) {
                                    QuantumResistantState.Auto -> ""
                                    QuantumResistantState.On -> LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG

                                    QuantumResistantState.Off -> LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
                                },
                        )
                    }

                VpnSettingItem.QuantumResistanceHeader ->
                    item(key = it.javaClass.simpleName) {
                        InformationComposeCell(
                            title = stringResource(R.string.quantum_resistant_title),
                            background =
                                if (initialScrollToFeature == FeatureIndicator.QUANTUM_RESISTANCE) {
                                    MaterialTheme.colorScheme.primary.copy(
                                        alpha = highlightAnimation.value
                                    )
                                } else {
                                    MaterialTheme.colorScheme.primary
                                },
                            onInfoClicked = navigateToQuantumResistanceInfo,
                            onCellClicked = navigateToQuantumResistanceInfo,
                            modifier = Modifier.animateItem(),
                        )
                    }

                VpnSettingItem.ServerIpOverridesHeader ->
                    item(key = it.javaClass.simpleName) {
                        ServerIpOverrides(navigateToServerIpOverrides, Modifier.animateItem())
                    }

                VpnSettingItem.Spacer ->
                    item(contentType = it.javaClass.simpleName) {
                        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing).animateItem())
                    }

                is VpnSettingItem.WireguardPortHeader ->
                    item(key = it.javaClass.simpleName) {
                        InformationComposeCell(
                            title = stringResource(id = R.string.wireguard_port_title),
                            onInfoClicked = { navigateToWireguardPortInfo(it.availablePortRanges) },
                            onCellClicked = { navigateToWireguardPortInfo(it.availablePortRanges) },
                            isEnabled = it.enabled,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.WireguardPortItem.Constraint ->
                    item(key = it.javaClass.simpleName + it.constraint) {
                        SelectableCell(
                            title =
                                when (it.constraint) {
                                    is Constraint.Only -> it.constraint.value.toString()

                                    is Constraint.Any -> stringResource(id = R.string.automatic)
                                },
                            testTag =
                                when (it.constraint) {
                                    is Constraint.Only ->
                                        String.format(
                                            null,
                                            LAZY_LIST_WIREGUARD_PORT_ITEM_X_TEST_TAG,
                                            it.constraint.value.value,
                                        )

                                    is Constraint.Any -> ""
                                },
                            isSelected = it.selected,
                            onCellClicked = { onWireguardPortSelected(it.constraint) },
                            isEnabled = it.enabled,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.WireguardPortItem.WireguardPortCustom ->
                    item(key = it.javaClass.simpleName) {
                        CustomPortCell(
                            title = stringResource(id = R.string.wireguard_custon_port_title),
                            isSelected = it.selected,
                            port = it.customPort,
                            onMainCellClicked = {
                                if (it.customPort != null) {
                                    onWireguardPortSelected(Constraint.Only(it.customPort))
                                } else {
                                    navigateToWireguardPortDialog(null, it.availablePortRanges)
                                }
                            },
                            onPortCellClicked = {
                                navigateToWireguardPortDialog(it.customPort, it.availablePortRanges)
                            },
                            isEnabled = it.enabled,
                            modifier = Modifier.animateItem(),
                            mainTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG,
                            numberTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG,
                        )
                    }

                VpnSettingItem.WireguardPortUnavailable ->
                    item(key = it.javaClass.simpleName) {
                        BaseSubtitleCell(
                            text =
                                stringResource(
                                    id = R.string.wg_port_subtitle,
                                    stringResource(R.string.wireguard),
                                ),
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.animateItem(),
                        )
                    }
            }
        }
    }
}

@Composable
private fun ServerIpOverrides(onServerIpOverridesClick: () -> Unit, modifier: Modifier = Modifier) {
    NavigationComposeCell(
        modifier = modifier,
        title = stringResource(id = R.string.server_ip_override),
        onClick = onServerIpOverridesClick,
    )
}

private fun VpnSettingsSideEffect.ShowToast.message(context: Context) =
    when (this) {
        VpnSettingsSideEffect.ShowToast.ApplySettingsWarning ->
            context.getString(R.string.settings_changes_effect_warning_short)
        VpnSettingsSideEffect.ShowToast.GenericError -> context.getString(R.string.error_occurred)
    }
