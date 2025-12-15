@file:OptIn(ExperimentalSharedTransitionApi::class, ExperimentalMaterial3Api::class)

package net.mullvad.mullvadvpn.compose.screen

import android.content.res.Resources
import android.os.Parcelable
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.AntiCensorshipSettingsDestination
import com.ramcosta.composedestinations.generated.destinations.AutoConnectAndLockdownModeDestination
import com.ramcosta.composedestinations.generated.destinations.ConnectOnStartupInfoDestination
import com.ramcosta.composedestinations.generated.destinations.ContentBlockersInfoDestination
import com.ramcosta.composedestinations.generated.destinations.CustomDnsInfoDestination
import com.ramcosta.composedestinations.generated.destinations.DeviceIpInfoDestination
import com.ramcosta.composedestinations.generated.destinations.DnsDestination
import com.ramcosta.composedestinations.generated.destinations.Ipv6InfoDestination
import com.ramcosta.composedestinations.generated.destinations.LocalNetworkSharingInfoDestination
import com.ramcosta.composedestinations.generated.destinations.MalwareInfoDestination
import com.ramcosta.composedestinations.generated.destinations.MtuDestination
import com.ramcosta.composedestinations.generated.destinations.QuantumResistanceInfoDestination
import com.ramcosta.composedestinations.generated.destinations.ServerIpOverridesDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import kotlinx.coroutines.launch
import kotlinx.parcelize.Parcelize
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.BaseSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.communication.DnsDialogResult
import net.mullvad.mullvadvpn.compose.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.preview.VpnSettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.VpnSettingItem
import net.mullvad.mullvadvpn.compose.state.VpnSettingsUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.OnNavResultValue
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.constant.SETTINGS_HIGHLIGHT_REPEAT_COUNT
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.DnsListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExpandableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.MtuListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.toTitle
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_AUTO_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV4_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV6_CELL_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.indexOfFirstOrNull
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsSideEffect
import net.mullvad.mullvadvpn.viewmodel.VpnSettingsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Default|NonDefault")
@Composable
private fun PreviewVpnSettings(
    @PreviewParameter(VpnSettingsUiStatePreviewParameterProvider::class)
    state: Lc<Boolean, VpnSettingsUiState>
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
            onSelectQuantumResistanceSetting = {},
            onToggleAutoStartAndConnectOnBoot = { _ -> },
            navigateToMalwareInfo = {},
            navigateToContentBlockersInfo = {},
            navigateToAutoConnectScreen = {},
            navigateToCustomDnsInfo = {},
            navigateToQuantumResistanceInfo = {},
            navigateToLocalNetworkSharingInfo = {},
            navigateToServerIpOverrides = {},
            onSelectDeviceIpVersion = {},
            onToggleIpv6 = {},
            onToggleContentBlockersExpanded = {},
            navigateToIpv6Info = {},
            navigateToDeviceIpInfo = {},
            navigateToConnectOnDeviceOnStartUpInfo = {},
            navigateToAntiCensorship = {},
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

    mtuDialogResult.OnNavResultValue { result ->
        if (!result) {
            vm.showGenericErrorToast()
        }
    }

    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            is VpnSettingsSideEffect.ShowToast ->
                launch {
                    snackbarHostState.showSnackbarImmediately(message = it.message(resources))
                }
            VpnSettingsSideEffect.NavigateToDnsDialog ->
                navigator.navigate(DnsDestination(null, null)) { launchSingleTop = true }
        }
    }

    val scrollToFeature = if (state.isScrollToFeatureEnabled()) navArgs.scrollToFeature else null

    VpnSettingsScreen(
        state = state,
        initialScrollToFeature = scrollToFeature,
        modifier =
            if (scrollToFeature != null) {
                Modifier.sharedBounds(
                    rememberSharedContentState(key = scrollToFeature),
                    animatedVisibilityScope = animatedVisibilityScope,
                )
            } else Modifier,
        snackbarHostState = snackbarHostState,
        navigateToContentBlockersInfo =
            dropUnlessResumed { navigator.navigate(ContentBlockersInfoDestination) },
        navigateToAutoConnectScreen =
            dropUnlessResumed { navigator.navigate(AutoConnectAndLockdownModeDestination) },
        navigateToCustomDnsInfo =
            dropUnlessResumed { navigator.navigate(CustomDnsInfoDestination) },
        navigateToMalwareInfo = dropUnlessResumed { navigator.navigate(MalwareInfoDestination) },
        navigateToQuantumResistanceInfo =
            dropUnlessResumed { navigator.navigate(QuantumResistanceInfoDestination) },
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
        onToggleDnsClick = vm::onToggleCustomDns,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onSelectQuantumResistanceSetting = vm::onSelectQuantumResistanceSetting,
        onToggleAutoStartAndConnectOnBoot = vm::onToggleAutoStartAndConnectOnBoot,
        onSelectDeviceIpVersion = vm::onDeviceIpVersionSelected,
        onToggleIpv6 = vm::setIpv6Enabled,
        navigateToIpv6Info = dropUnlessResumed { navigator.navigate(Ipv6InfoDestination) },
        navigateToDeviceIpInfo = dropUnlessResumed { navigator.navigate(DeviceIpInfoDestination) },
        navigateToConnectOnDeviceOnStartUpInfo =
            dropUnlessResumed { navigator.navigate(ConnectOnStartupInfoDestination) },
        navigateToAntiCensorship =
            dropUnlessResumed { navigator.navigate(AntiCensorshipSettingsDestination()) },
    )
}

@Suppress("LongParameterList")
@Composable
fun VpnSettingsScreen(
    state: Lc<Boolean, VpnSettingsUiState>,
    initialScrollToFeature: FeatureIndicator?,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    navigateToContentBlockersInfo: () -> Unit,
    navigateToAutoConnectScreen: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToAntiCensorship: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
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
    onSelectQuantumResistanceSetting: (Boolean) -> Unit,
    onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit,
    onSelectDeviceIpVersion: (ipVersion: Constraint<IpVersion>) -> Unit,
    onToggleIpv6: (Boolean) -> Unit,
    navigateToIpv6Info: () -> Unit,
    navigateToDeviceIpInfo: () -> Unit,
    navigateToConnectOnDeviceOnStartUpInfo: () -> Unit,
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
                    if (state.isModal()) {
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
            Box(modifier = Modifier.fillMaxSize().padding(it)) {
                when (state) {
                    is Lc.Loading -> CircularProgressIndicator(modifier.align(Alignment.Center))

                    is Lc.Content ->
                        VpnSettingsContent(
                            state.value,
                            initialScrollToFeature,
                            canScroll,
                            navigateToContentBlockersInfo,
                            navigateToAutoConnectScreen,
                            navigateToCustomDnsInfo,
                            navigateToMalwareInfo,
                            navigateToQuantumResistanceInfo,
                            navigateToLocalNetworkSharingInfo,
                            navigateToServerIpOverrides,
                            navigateToAntiCensorship,
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
                            onSelectQuantumResistanceSetting,
                            onToggleAutoStartAndConnectOnBoot,
                            onSelectDeviceIpVersion,
                            onToggleIpv6,
                            navigateToIpv6Info,
                            navigateToDeviceIpInfo,
                            navigateToConnectOnDeviceOnStartUpInfo,
                        )
                }
            }
        },
    )
}

@Suppress("LongMethod", "LongParameterList", "CyclomaticComplexMethod")
@Composable
fun VpnSettingsContent(
    state: VpnSettingsUiState,
    initialScrollToFeature: FeatureIndicator?,
    canScroll: MutableState<Boolean>,
    navigateToContentBlockersInfo: () -> Unit,
    navigateToAutoConnectScreen: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    navigateToAntiCensorship: () -> Unit,
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
    onSelectQuantumResistanceSetting: (Boolean) -> Unit,
    onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit,
    onSelectDeviceIpVersion: (ipVersion: Constraint<IpVersion>) -> Unit,
    onToggleIpv6: (Boolean) -> Unit,
    navigateToIpv6Info: () -> Unit,
    navigateToDeviceIpInfo: () -> Unit,
    navigateToConnectOnDeviceOnStartUpInfo: () -> Unit,
) {
    val initialIndexFocus =
        when (initialScrollToFeature) {
            FeatureIndicator.LAN_SHARING -> VpnSettingItem.LocalNetworkSharingSetting::class
            FeatureIndicator.QUANTUM_RESISTANCE -> VpnSettingItem.QuantumResistantSetting::class
            FeatureIndicator.DNS_CONTENT_BLOCKERS -> VpnSettingItem.DnsContentBlockersHeader::class
            FeatureIndicator.CUSTOM_MTU -> VpnSettingItem.Mtu::class
            FeatureIndicator.CUSTOM_DNS -> VpnSettingItem.CustomDnsServerSetting::class
            else -> null
        }?.let { clazz -> state.settings.indexOfFirstOrNull { it::class == clazz } } ?: 0

    val highlightAnimation = remember { Animatable(AlphaVisible) }
    if (initialScrollToFeature != null) {
        LaunchedEffect(Unit) {
            repeat(times = SETTINGS_HIGHLIGHT_REPEAT_COUNT) {
                highlightAnimation.animateTo(AlphaInvisible)
                highlightAnimation.animateTo(AlphaVisible)
            }
        }
    }

    @Composable
    fun highlightBackgroundAlpha(featureIndicator: FeatureIndicator): Float =
        if (initialScrollToFeature == featureIndicator) {
            highlightAnimation.value
        } else {
            1.0f
        }

    val lazyListState = rememberLazyListState(initialIndexFocus)
    canScroll.value = lazyListState.canScrollForward || lazyListState.canScrollBackward
    val focusRequesters: Map<FeatureIndicator, FocusRequester> = remember {
        featureIndicators().associateWith { FocusRequester() }
    }
    if (initialScrollToFeature != null) {
        LaunchedEffect(Unit) { focusRequesters[initialScrollToFeature]?.requestFocus() }
    }
    LazyColumn(
        modifier =
            Modifier.testTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .fillMaxSize()
                .drawVerticalScrollbar(
                    state = lazyListState,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .padding(horizontal = Dimens.sideMarginNew)
                .animateContentSize(),
        state = lazyListState,
    ) {
        state.settings.forEach {
            when (it) {
                VpnSettingItem.AutoConnectAndLockdownMode ->
                    item(key = it::class.simpleName) {
                        NavigationListItem(
                            modifier = Modifier.animateItem(),
                            title = stringResource(id = R.string.auto_connect_and_lockdown_mode),
                            onClick = { navigateToAutoConnectScreen() },
                        )
                    }

                VpnSettingItem.AutoConnectAndLockdownModeInfo ->
                    item(key = it::class.simpleName) {
                        SwitchComposeSubtitleCell(
                            modifier = Modifier.animateItem(),
                            text =
                                stringResource(id = R.string.auto_connect_and_lockdown_mode_footer),
                        )
                    }

                is VpnSettingItem.ConnectDeviceOnStartUpSetting ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.connect_on_start),
                            isToggled = it.enabled,
                            onInfoClicked = navigateToConnectOnDeviceOnStartUpInfo,
                            onCellClicked = { newValue ->
                                onToggleAutoStartAndConnectOnBoot(newValue)
                            },
                        )
                    }

                VpnSettingItem.CustomDnsAdd ->
                    item(key = it::class.simpleName) {
                        MullvadListItem(
                            modifier = Modifier.animateItem(),
                            hierarchy = Hierarchy.Child1,
                            position = Position.Bottom,
                            onClick = { navigateToDns(null, null) },
                            content = { Text(text = stringResource(id = R.string.add_a_server)) },
                            trailingContent = {
                                DividerButton(
                                    onClick = { navigateToDns(null, null) },
                                    icon = Icons.Default.Add,
                                )
                            },
                        )
                    }

                is VpnSettingItem.CustomDnsEntry ->
                    item(key = it::class.simpleName + it.index) {
                        DnsListItem(
                            modifier = Modifier.animateItem(),
                            hierarchy = Hierarchy.Child1,
                            position = Position.Middle,
                            address = it.customDnsItem.address,
                            isUnreachableLocalDnsWarningVisible = it.showUnreachableLocalDnsWarning,
                            isUnreachableIpv6DnsWarningVisible = it.showUnreachableIpv6DnsWarning,
                            onClick = { navigateToDns(it.index, it.customDnsItem.address) },
                        )
                    }

                VpnSettingItem.CustomDnsInfo ->
                    item(key = it::class.simpleName) {
                        BaseSubtitleCell(
                            modifier = Modifier.animateItem(),
                            text = textResource(id = R.string.custom_dns_footer),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                is VpnSettingItem.CustomDnsServerSetting ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier =
                                Modifier.animateItem()
                                    .focusRequester(
                                        focusRequesters.getValue(FeatureIndicator.CUSTOM_DNS)
                                    ),
                            position = if (it.enabled) Position.Top else Position.Single,
                            title = stringResource(R.string.enable_custom_dns),
                            isToggled = it.enabled,
                            isEnabled = it.isOptionEnabled,
                            onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                            onInfoClicked = { navigateToCustomDnsInfo() },
                            backgroundAlpha = highlightBackgroundAlpha(FeatureIndicator.CUSTOM_DNS),
                        )
                    }
                VpnSettingItem.CustomDnsUnavailable ->
                    item(key = it::class.simpleName) {
                        BaseSubtitleCell(
                            modifier = Modifier.animateItem(),
                            text =
                                textResource(
                                    id = R.string.custom_dns_disable_mode_subtitle,
                                    textResource(id = R.string.dns_content_blockers),
                                ),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }

                VpnSettingItem.DeviceIpVersionHeader ->
                    item(key = it::class.simpleName) {
                        InfoListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Top,
                            title = stringResource(R.string.device_ip_version_title),
                            onInfoClicked = navigateToDeviceIpInfo,
                            onCellClicked = navigateToDeviceIpInfo,
                        )
                    }

                is VpnSettingItem.DeviceIpVersionItem ->
                    item(key = it::class.simpleName + it.constraint.getOrNull().toString()) {
                        SelectableListItem(
                            modifier = Modifier.animateItem(),
                            hierarchy = Hierarchy.Child1,
                            position =
                                if (
                                    it.constraint is Constraint.Only &&
                                        it.constraint.value == IpVersion.IPV6
                                ) {
                                    Position.Bottom
                                } else Position.Middle,
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
                            onClick = { onSelectDeviceIpVersion(it.constraint) },
                            testTag =
                                when (it.constraint.getOrNull()) {
                                    null -> WIREGUARD_DEVICE_IP_AUTO_CELL_TEST_TAG
                                    IpVersion.IPV4 -> WIREGUARD_DEVICE_IP_IPV4_CELL_TEST_TAG
                                    IpVersion.IPV6 -> WIREGUARD_DEVICE_IP_IPV6_CELL_TEST_TAG
                                },
                        )
                    }

                VpnSettingItem.Divider -> {
                    item(contentType = it::class.simpleName) {
                        HorizontalDivider(
                            modifier = Modifier.animateItem(),
                            color = Color.Transparent,
                        )
                    }
                }

                is VpnSettingItem.DnsContentBlockerItem.Ads ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Middle,
                            hierarchy = Hierarchy.Child1,
                            title = stringResource(R.string.block_ads_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockAds(it) },
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.AdultContent ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Middle,
                            hierarchy = Hierarchy.Child1,
                            title = stringResource(R.string.block_adult_content_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockAdultContent(it) },
                        )
                    }
                is VpnSettingItem.DnsContentBlockerItem.Gambling ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Middle,
                            hierarchy = Hierarchy.Child1,
                            title = stringResource(R.string.block_gambling_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockGambling(it) },
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.Malware ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Middle,
                            hierarchy = Hierarchy.Child1,
                            title = stringResource(R.string.block_malware_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockMalware(it) },
                            onInfoClicked = { navigateToMalwareInfo() },
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.SocialMedia ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Bottom,
                            hierarchy = Hierarchy.Child1,
                            title = stringResource(R.string.block_social_media_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockSocialMedia(it) },
                        )
                    }

                is VpnSettingItem.DnsContentBlockerItem.Trackers ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.block_trackers_title),
                            isToggled = it.enabled,
                            isEnabled = it.featureEnabled,
                            onCellClicked = { onToggleBlockTrackers(it) },
                            hierarchy = Hierarchy.Child1,
                            position = Position.Middle,
                        )
                    }

                is VpnSettingItem.DnsContentBlockersHeader ->
                    item(key = it::class.simpleName) {
                        ExpandableListItem(
                            modifier =
                                Modifier.animateItem()
                                    .focusRequester(
                                        focusRequesters.getValue(
                                            FeatureIndicator.DNS_CONTENT_BLOCKERS
                                        )
                                    ),
                            position = if (it.expanded) Position.Top else Position.Single,
                            title = stringResource(R.string.dns_content_blockers),
                            isExpanded = it.expanded,
                            isEnabled = it.featureEnabled,
                            onInfoClicked = { navigateToContentBlockersInfo() },
                            onCellClicked = { onToggleContentBlockersExpanded() },
                        )
                    }

                VpnSettingItem.DnsContentBlockersUnavailable ->
                    item(key = it::class.simpleName) {
                        BaseSubtitleCell(
                            text =
                                stringResource(
                                    id = R.string.dns_content_blockers_subtitle,
                                    stringResource(id = R.string.enable_custom_dns),
                                ),
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.animateItem(),
                        )
                    }

                is VpnSettingItem.EnableIpv6Setting ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            title = stringResource(R.string.enable_ipv6),
                            isToggled = it.enabled,
                            isEnabled = true,
                            onCellClicked = onToggleIpv6,
                            onInfoClicked = navigateToIpv6Info,
                        )
                    }

                is VpnSettingItem.LocalNetworkSharingSetting ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier =
                                Modifier.animateItem()
                                    .focusRequester(
                                        focusRequesters.getValue(FeatureIndicator.LAN_SHARING)
                                    ),
                            backgroundAlpha =
                                highlightBackgroundAlpha(FeatureIndicator.LAN_SHARING),
                            title = stringResource(R.string.local_network_sharing),
                            isToggled = it.enabled,
                            isEnabled = true,
                            onCellClicked = { newValue -> onToggleLocalNetworkSharing(newValue) },
                            onInfoClicked = navigateToLocalNetworkSharingInfo,
                        )
                    }

                is VpnSettingItem.Mtu ->
                    item(key = it::class.simpleName) {
                        MtuListItem(
                            modifier =
                                Modifier.animateItem()
                                    .focusRequester(
                                        focusRequesters.getValue(FeatureIndicator.CUSTOM_MTU)
                                    )
                                    .testTag(LAZY_LIST_LAST_ITEM_TEST_TAG),
                            mtuValue = it.mtu,
                            onEditMtu = { navigateToMtuDialog(it.mtu) },
                            backgroundAlpha = highlightBackgroundAlpha(FeatureIndicator.CUSTOM_MTU),
                        )
                    }

                is VpnSettingItem.QuantumResistantSetting ->
                    item(key = it::class.simpleName) {
                        SwitchListItem(
                            modifier = Modifier.animateItem(),
                            position = Position.Single,
                            hierarchy = Hierarchy.Parent,
                            title = stringResource(R.string.quantum_resistant_title),
                            isToggled = it.enabled,
                            onInfoClicked = navigateToQuantumResistanceInfo,
                            onCellClicked = onSelectQuantumResistanceSetting,
                            backgroundAlpha =
                                highlightBackgroundAlpha(FeatureIndicator.QUANTUM_RESISTANCE),
                            testTag = LAZY_LIST_QUANTUM_ITEM_TEST_TAG,
                        )
                    }

                VpnSettingItem.ServerIpOverrides ->
                    item(key = it::class.simpleName) {
                        ServerIpOverrides(navigateToServerIpOverrides, Modifier.animateItem())
                    }

                VpnSettingItem.AntiCensorshipHeader ->
                    item(key = it::class.simpleName) {
                        NavigationListItem(
                            modifier =
                                Modifier.testTag(LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG)
                                    .animateItem(),
                            title = stringResource(id = R.string.anti_censorship),
                            subtitle = state.obfuscationMode.toTitle(),
                            onClick = navigateToAntiCensorship,
                        )
                    }

                VpnSettingItem.Spacer ->
                    item(contentType = it::class.simpleName) {
                        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing).animateItem())
                    }
            }
        }
    }
}

@Composable
private fun ServerIpOverrides(onServerIpOverridesClick: () -> Unit, modifier: Modifier = Modifier) {
    NavigationListItem(
        title = stringResource(id = R.string.server_ip_override),
        modifier = modifier,
        onClick = onServerIpOverridesClick,
        testTag = SERVER_IP_OVERRIDE_BUTTON_TEST_TAG,
    )
}

private fun VpnSettingsSideEffect.ShowToast.message(resources: Resources) =
    when (this) {
        VpnSettingsSideEffect.ShowToast.ApplySettingsWarning ->
            resources.getString(R.string.settings_changes_effect_warning_short)
        VpnSettingsSideEffect.ShowToast.GenericError -> resources.getString(R.string.error_occurred)
    }

private fun Lc<Boolean, VpnSettingsUiState>.isModal() =
    when (this) {
        is Lc.Loading -> value
        is Lc.Content -> value.isModal
    }

private fun Lc<Boolean, VpnSettingsUiState>.isScrollToFeatureEnabled() =
    when (this) {
        is Lc.Loading -> value
        is Lc.Content -> value.isScrollToFeatureEnabled
    }

// A list of feature indicators on this screen
private fun featureIndicators() =
    listOf(
        FeatureIndicator.UDP_2_TCP,
        FeatureIndicator.SHADOWSOCKS,
        FeatureIndicator.QUIC,
        FeatureIndicator.LWO,
        FeatureIndicator.LAN_SHARING,
        FeatureIndicator.QUANTUM_RESISTANCE,
        FeatureIndicator.DNS_CONTENT_BLOCKERS,
        FeatureIndicator.CUSTOM_MTU,
        FeatureIndicator.CUSTOM_DNS,
    )
