@file:OptIn(ExperimentalSharedTransitionApi::class)

package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import android.os.Parcelable
import androidx.compose.animation.Animatable
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
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
import kotlinx.parcelize.Parcelize
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
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
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
            navigateToWireguardPortDialog = { a, b -> },
            navigateToServerIpOverrides = {},
            onSelectDeviceIpVersion = {},
            onToggleIpv6Toggle = {},
            onToggleContentBlockersExpanded = {},
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
        onToggleIpv6Toggle = vm::setIpv6Enabled,
    )
}

@Suppress("LongMethod", "LongParameterList")
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
    onToggleIpv6Toggle: (Boolean) -> Unit,
) {
    val topPadding = 6.dp

    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.settings_vpn),
        modifier = modifier,
        navigationIcon = {
            if (state.isModal) {
                NavigateCloseIconButton(onNavigateClose = onBackClick)
            } else {
                NavigateBackIconButton(onNavigateBack = onBackClick)
            }
        },
        snackbarHostState = snackbarHostState,
    ) {
        when (state) {
            is VpnSettingsUiState.Loading -> CircularProgressIndicator()
            is VpnSettingsUiState.Content -> {
                val initialIndexFocus =
                    when (initialScrollToFeature) {
                        FeatureIndicator.UDP_2_TCP,
                        FeatureIndicator.SHADOWSOCKS -> VpnSettingItem.ObfuscationHeader::class
                        FeatureIndicator.LAN_SHARING ->
                            VpnSettingItem.LocalNetworkSharingHeader::class
                        FeatureIndicator.QUANTUM_RESISTANCE ->
                            VpnSettingItem.QuantumResistanceHeader::class
                        FeatureIndicator.DNS_CONTENT_BLOCKERS ->
                            VpnSettingItem.DnsContentBlockers::class
                        FeatureIndicator.CUSTOM_MTU -> VpnSettingItem.MtuHeader::class
                        else -> null
                    }?.let { clazz ->
                        state.settings.indexOfFirstOrNull { it::class == clazz } ?: 0
                    } ?: 0

                val animatable = remember { androidx.compose.animation.core.Animatable(1f) }
                if (initialScrollToFeature != null) {
                    LaunchedEffect(Unit) {
                        animatable.animateTo(0f)
                        animatable.animateTo(1f)
                        animatable.animateTo(0f)
                        animatable.animateTo(1f)
                        animatable.animateTo(0f)
                        animatable.animateTo(1f)
                    }
                }

                LazyColumn(
                    modifier = it.testTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG).animateContentSize(),
                    state = rememberLazyListState(initialIndexFocus),
                ) {
                    items(state.settings) {
                        when (it) {
                            VpnSettingItem.AutoConnectAndLockdownModeHeader ->
                                NavigationComposeCell(
                                    title =
                                        stringResource(
                                            id = R.string.auto_connect_and_lockdown_mode
                                        ),
                                    onClick = { navigateToAutoConnectScreen() },
                                )

                            VpnSettingItem.AutoConnectAndLockdownModeInfo ->
                                SwitchComposeSubtitleCell(
                                    text =
                                        stringResource(
                                            id = R.string.auto_connect_and_lockdown_mode_footer
                                        )
                                )

                            is VpnSettingItem.ConnectDeviceOnStartUpHeader ->
                                HeaderSwitchComposeCell(
                                    title = stringResource(R.string.connect_on_start),
                                    isToggled = it.enabled,
                                    onCellClicked = { newValue ->
                                        onToggleAutoStartAndConnectOnBoot(newValue)
                                    },
                                )

                            VpnSettingItem.ConnectDeviceOnStartUpInfo ->
                                SwitchComposeSubtitleCell(
                                    text =
                                        textResource(
                                            R.string.connect_on_start_footer,
                                            textResource(R.string.auto_connect_and_lockdown_mode),
                                        )
                                )

                            VpnSettingItem.CustomDnsAdd ->
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
                                    startPadding = Dimens.cellStartPaddingLarge,
                                )

                            is VpnSettingItem.CustomDnsEntry ->
                                DnsCell(
                                    address = it.customDnsItem.address,
                                    isUnreachableLocalDnsWarningVisible =
                                        it.showUnreachableLocalDnsWarning,
                                    isUnreachableIpv6DnsWarningVisible =
                                        it.showUnreachableIpv6DnsWarning,
                                    onClick = { navigateToDns(it.index, it.customDnsItem.address) },
                                    modifier = Modifier.animateItem(),
                                )

                            VpnSettingItem.CustomDnsInfo ->
                                CustomDnsCellSubtitle(
                                    isCellClickable = true,
                                    modifier =
                                        Modifier.padding(
                                            start = Dimens.cellStartPadding,
                                            top = topPadding,
                                            end = Dimens.cellEndPadding,
                                            bottom = Dimens.cellVerticalSpacing,
                                        ),
                                )

                            is VpnSettingItem.CustomDnsServerHeader ->
                                HeaderSwitchComposeCell(
                                    title = stringResource(R.string.enable_custom_dns),
                                    isToggled = it.enabled,
                                    isEnabled = it.isOptionEnabled,
                                    onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                                    onInfoClicked = { navigateToCustomDnsInfo() },
                                )
                            // TODO Merge with CustomDnsInfo
                            VpnSettingItem.CustomDnsUnavailable ->
                                CustomDnsCellSubtitle(
                                    isCellClickable = false,
                                    modifier =
                                        Modifier.padding(
                                            start = Dimens.cellStartPadding,
                                            top = topPadding,
                                            end = Dimens.cellEndPadding,
                                            bottom = Dimens.cellVerticalSpacing,
                                        ),
                                )

                            VpnSettingItem.DeviceIpVersionHeader ->
                                InformationComposeCell(
                                    title = stringResource(R.string.device_ip_version_title)
                                )

                            is VpnSettingItem.DeviceIpVersionItem ->
                                SelectableCell(
                                    title =
                                        when (it.constraint) {
                                            Constraint.Any ->
                                                stringResource(id = R.string.automatic)
                                            is Constraint.Only ->
                                                when (it.constraint.value) {
                                                    IpVersion.IPV4 ->
                                                        stringResource(id = R.string.ipv4)
                                                    IpVersion.IPV6 ->
                                                        stringResource(id = R.string.ipv6)
                                                }
                                        },
                                    isSelected = it.selected,
                                    onCellClicked = { onSelectDeviceIpVersion(it.constraint) },
                                )

                            VpnSettingItem.Divider -> HorizontalDivider(color = Color.Transparent)
                            is VpnSettingItem.DnsContentBlockerItem.Ads ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_ads_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockAds(it) },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockerItem.AdultContent ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_adult_content_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockAdultContent(it) },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockerItem.Gambling ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_gambling_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockGambling(it) },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockerItem.Malware ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_malware_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockMalware(it) },
                                    onInfoClicked = { navigateToMalwareInfo() },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockerItem.SocialMedia ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_social_media_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockSocialMedia(it) },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockerItem.Trackers ->
                                NormalSwitchComposeCell(
                                    title = stringResource(R.string.block_trackers_title),
                                    isToggled = it.enabled,
                                    isEnabled = it.featureEnabled,
                                    onCellClicked = { onToggleBlockTrackers(it) },
                                    background = MaterialTheme.colorScheme.surfaceContainerLow,
                                    startPadding = Dimens.indentedCellStartPadding,
                                )

                            is VpnSettingItem.DnsContentBlockers ->
                                ExpandableComposeCell(
                                    title = stringResource(R.string.dns_content_blockers),
                                    background =
                                        if (
                                            initialScrollToFeature ==
                                                FeatureIndicator.DNS_CONTENT_BLOCKERS
                                        ) {
                                            MaterialTheme.colorScheme.primary.copy(
                                                alpha = animatable.value
                                            )
                                        } else {
                                            MaterialTheme.colorScheme.primary
                                        },
                                    isExpanded = it.expanded,
                                    isEnabled = it.featureEnabled,
                                    onInfoClicked = { navigateToContentBlockersInfo() },
                                    onCellClicked = { onToggleContentBlockersExpanded() },
                                )

                            VpnSettingItem.DnsContentBlockersUnavailable ->
                                ContentBlockersDisableModeCellSubtitle(
                                    Modifier.background(MaterialTheme.colorScheme.surface)
                                        .padding(
                                            start = Dimens.cellStartPadding,
                                            top = topPadding,
                                            end = Dimens.cellEndPadding,
                                            bottom = Dimens.cellVerticalSpacing,
                                        )
                                )

                            is VpnSettingItem.EnableIpv6Header ->
                                HeaderSwitchComposeCell(
                                    title = stringResource(R.string.enable_ipv6),
                                    isToggled = it.enabled,
                                    isEnabled = true,
                                    onCellClicked = { newValue -> onToggleIpv6Toggle(newValue) },
                                )

                            is VpnSettingItem.LocalNetworkSharingHeader ->
                                HeaderSwitchComposeCell(
                                    title = stringResource(R.string.local_network_sharing),
                                    isToggled = it.enabled,
                                    isEnabled = true,
                                    onCellClicked = { newValue ->
                                        onToggleLocalNetworkSharing(newValue)
                                    },
                                    onInfoClicked = navigateToLocalNetworkSharingInfo,
                                )

                            is VpnSettingItem.MtuHeader ->
                                MtuComposeCell(
                                    mtuValue = it.mtu,
                                    onEditMtu = { navigateToMtuDialog(it.mtu) },
                                )

                            VpnSettingItem.MtuInfo ->
                                MtuSubtitle(
                                    modifier = Modifier.testTag(LAZY_LIST_LAST_ITEM_TEST_TAG)
                                )

                            VpnSettingItem.ObfuscationHeader ->
                                InformationComposeCell(
                                    title = stringResource(R.string.obfuscation_title),
                                    onInfoClicked = navigateToObfuscationInfo,
                                    onCellClicked = navigateToObfuscationInfo,
                                    testTag = LAZY_LIST_WIREGUARD_OBFUSCATION_TITLE_TEST_TAG,
                                )

                            is VpnSettingItem.ObfuscationItem.Automatic ->
                                SelectableCell(
                                    title = stringResource(id = R.string.automatic),
                                    isSelected = it.selected,
                                    onCellClicked = {
                                        onSelectObfuscationMode(ObfuscationMode.Auto)
                                    },
                                )

                            is VpnSettingItem.ObfuscationItem.Off ->
                                SelectableCell(
                                    title = stringResource(id = R.string.off),
                                    isSelected = it.selected,
                                    onCellClicked = {
                                        onSelectObfuscationMode(ObfuscationMode.Off)
                                    },
                                    testTag = WIREGUARD_OBFUSCATION_OFF_CELL,
                                )

                            is VpnSettingItem.ObfuscationItem.Shadowsocks ->
                                ObfuscationModeCell(
                                    obfuscationMode = ObfuscationMode.Shadowsocks,
                                    isSelected = it.selected,
                                    port = it.port,
                                    onSelected = onSelectObfuscationMode,
                                    onNavigate = navigateToShadowSocksSettings,
                                    testTag = WIREGUARD_OBFUSCATION_SHADOWSOCKS_CELL,
                                )

                            is VpnSettingItem.ObfuscationItem.UdpOverTcp ->
                                ObfuscationModeCell(
                                    obfuscationMode = ObfuscationMode.Udp2Tcp,
                                    isSelected = it.selected,
                                    port = it.port,
                                    onSelected = onSelectObfuscationMode,
                                    onNavigate = navigateToUdp2TcpSettings,
                                    testTag = WIREGUARD_OBFUSCATION_UDP_OVER_TCP_CELL,
                                )

                            is VpnSettingItem.QuantumItem ->
                                SelectableCell(
                                    title =
                                        when (it.quantumResistantState) {
                                            QuantumResistantState.Auto ->
                                                stringResource(id = R.string.automatic)
                                            QuantumResistantState.Off ->
                                                stringResource(id = R.string.off)
                                            QuantumResistantState.On ->
                                                stringResource(id = R.string.on)
                                        },
                                    isSelected = it.selected,
                                    onCellClicked = {
                                        onSelectQuantumResistanceSetting(it.quantumResistantState)
                                    },
                                    testTag =
                                        when (it.quantumResistantState) {
                                            QuantumResistantState.Auto -> ""
                                            QuantumResistantState.On ->
                                                LAZY_LIST_QUANTUM_ITEM_ON_TEST_TAG
                                            QuantumResistantState.Off ->
                                                LAZY_LIST_QUANTUM_ITEM_OFF_TEST_TAG
                                        },
                                )

                            VpnSettingItem.QuantumResistanceHeader ->
                                InformationComposeCell(
                                    title = stringResource(R.string.quantum_resistant_title),
                                    onInfoClicked = navigateToQuantumResistanceInfo,
                                    onCellClicked = navigateToQuantumResistanceInfo,
                                )

                            VpnSettingItem.ServerIpOverridesHeader ->
                                ServerIpOverrides(navigateToServerIpOverrides)

                            VpnSettingItem.Spacer ->
                                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))

                            is VpnSettingItem.WireguardPortHeader ->
                                InformationComposeCell(
                                    title = stringResource(id = R.string.wireguard_port_title),
                                    onInfoClicked = {
                                        navigateToWireguardPortInfo(it.availablePortRanges)
                                    },
                                    onCellClicked = {
                                        navigateToWireguardPortInfo(it.availablePortRanges)
                                    },
                                    isEnabled = it.enabled,
                                )
                            is VpnSettingItem.WireguardPortItem.Constraint ->
                                SelectableCell(
                                    title =
                                        when (it.constraint) {
                                            is Constraint.Only -> it.constraint.value.toString()
                                            is Constraint.Any ->
                                                stringResource(id = R.string.automatic)
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
                                )

                            is VpnSettingItem.WireguardPortItem.WireguardPortCustom ->
                                CustomPortCell(
                                    title =
                                        stringResource(id = R.string.wireguard_custon_port_title),
                                    isSelected = it.selected,
                                    port = it.customPort,
                                    onMainCellClicked = {
                                        if (it.customPort != null) {
                                            onWireguardPortSelected(Constraint.Only(it.customPort))
                                        } else {
                                            navigateToWireguardPortDialog(
                                                it.customPort,
                                                it.availablePortRanges,
                                            )
                                        }
                                    },
                                    onPortCellClicked = {
                                        navigateToWireguardPortDialog(
                                            it.customPort,
                                            it.availablePortRanges,
                                        )
                                    },
                                    isEnabled = it.enabled,
                                    mainTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_TEXT_TEST_TAG,
                                    numberTestTag = LAZY_LIST_WIREGUARD_CUSTOM_PORT_NUMBER_TEST_TAG,
                                )

                            VpnSettingItem.WireguardPortUnavailable ->
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
                }
            }
        }
    }
}

@Composable
private fun ServerIpOverrides(onServerIpOverridesClick: () -> Unit) {
    NavigationComposeCell(
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
