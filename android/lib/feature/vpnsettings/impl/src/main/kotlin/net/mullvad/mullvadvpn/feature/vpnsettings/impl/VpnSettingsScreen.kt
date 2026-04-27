@file:OptIn(ExperimentalSharedTransitionApi::class, ExperimentalMaterial3Api::class)

package net.mullvad.mullvadvpn.feature.vpnsettings.impl

import android.content.res.Resources
import androidx.activity.compose.BackHandler
import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.ExperimentalSharedTransitionApi
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
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
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.RunOnKeyChange
import net.mullvad.mullvadvpn.common.compose.SETTINGS_HIGHLIGHT_REPEAT_COUNT
import net.mullvad.mullvadvpn.common.compose.assureHasDetailPane
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.navigateReplaceIfDetailPane
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.AntiCensorshipNavKey
import net.mullvad.mullvadvpn.feature.autoconnect.api.AutoConnectNavKey
import net.mullvad.mullvadvpn.feature.dns.api.DnsSettingsNavKey
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ServerIpOverrideNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.ConnectOnStartupInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.DeviceIpInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.Ipv6InfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.LocalNetworkSharingInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.MtuNavResult
import net.mullvad.mullvadvpn.feature.vpnsettings.api.QuantumResistanceInfoNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.VpnSettingsNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.util.indexOfFirstOrNull
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.model.IpVersion
import net.mullvad.mullvadvpn.lib.model.Mtu
import net.mullvad.mullvadvpn.lib.ui.component.MullvadSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.MtuListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.toTitle
import net.mullvad.mullvadvpn.lib.ui.component.text.ListItemInfo
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_ANTI_CENSORSHIP_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_DNS_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_LAST_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_QUANTUM_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_VPN_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDE_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_AUTO_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV4_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.WIREGUARD_DEVICE_IP_IPV6_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaVisible
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

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
            onToggleLocalNetworkSharing = {},
            navigateToMtuDialog = {},
            navigateToDns = {},
            onBackClick = {},
            onSelectQuantumResistanceSetting = {},
            onToggleAutoStartAndConnectOnBoot = { _ -> },
            navigateToAutoConnectScreen = {},
            navigateToQuantumResistanceInfo = {},
            navigateToLocalNetworkSharingInfo = {},
            navigateToServerIpOverrides = {},
            onSelectDeviceIpVersion = {},
            onToggleIpv6 = {},
            navigateToIpv6Info = {},
            navigateToDeviceIpInfo = {},
            navigateToConnectOnDeviceOnStartUpInfo = {},
            navigateToAntiCensorship = {},
        )
    }
}

@Composable
@Suppress("LongMethod")
fun SharedTransitionScope.VpnSettings(
    navArgs: VpnSettingsNavKey,
    navigator: Navigator,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val vm = koinViewModel<VpnSettingsViewModel> { parametersOf(navArgs) }
    val state by vm.uiState.collectAsStateWithLifecycle()
    val resultStore = LocalResultStore.current

    BackHandler(enabled = navigator.screenIsListDetailTargetWidth) {
        navigator.goBackUntil(VpnSettingsNavKey(), inclusive = true)
    }

    navigator.assureHasDetailPane<VpnSettingsNavKey>(AutoConnectNavKey)

    resultStore.consumeResult<MtuNavResult> { result ->
        if (!result.complete) {
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
        }
    }

    val scrollToFeature = navArgs.scrollToFeature

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
        navigateToAutoConnectScreen =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(AutoConnectNavKey) },
        navigateToQuantumResistanceInfo =
            dropUnlessResumed { navigator.navigate(QuantumResistanceInfoNavKey) },
        navigateToLocalNetworkSharingInfo =
            dropUnlessResumed { navigator.navigate(LocalNetworkSharingInfoNavKey) },
        navigateToServerIpOverrides =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(ServerIpOverrideNavKey()) },
        onToggleLocalNetworkSharing = vm::onToggleLocalNetworkSharing,
        navigateToMtuDialog = dropUnlessResumed { mtu: Mtu? -> navigator.navigate(MtuNavKey(mtu)) },
        navigateToDns = dropUnlessResumed { navigator.navigate(DnsSettingsNavKey()) },
        onSelectQuantumResistanceSetting = vm::onSelectQuantumResistanceSetting,
        onToggleAutoStartAndConnectOnBoot = vm::onToggleAutoStartAndConnectOnBoot,
        onSelectDeviceIpVersion = vm::onDeviceIpVersionSelected,
        onToggleIpv6 = vm::setIpv6Enabled,
        navigateToIpv6Info = dropUnlessResumed { navigator.navigate(Ipv6InfoNavKey) },
        navigateToDeviceIpInfo = dropUnlessResumed { navigator.navigate(DeviceIpInfoNavKey) },
        navigateToConnectOnDeviceOnStartUpInfo =
            dropUnlessResumed { navigator.navigate(ConnectOnStartupInfoNavKey) },
        navigateToAntiCensorship =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(AntiCensorshipNavKey()) },
        onBackClick =
            dropUnlessResumed { navigator.goBackUntil(VpnSettingsNavKey(), inclusive = true) },
    )
}

@Suppress("LongParameterList")
@Composable
fun VpnSettingsScreen(
    state: Lc<Boolean, VpnSettingsUiState>,
    initialScrollToFeature: FeatureIndicator?,
    modifier: Modifier = Modifier,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    navigateToAutoConnectScreen: () -> Unit,
    navigateToAntiCensorship: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    onToggleLocalNetworkSharing: (Boolean) -> Unit,
    navigateToMtuDialog: (mtu: Mtu?) -> Unit,
    navigateToDns: () -> Unit,
    onBackClick: () -> Unit,
    onSelectQuantumResistanceSetting: (Boolean) -> Unit,
    onToggleAutoStartAndConnectOnBoot: (Boolean) -> Unit,
    onSelectDeviceIpVersion: (ipVersion: Constraint<IpVersion>) -> Unit,
    onToggleIpv6: (Boolean) -> Unit,
    navigateToIpv6Info: () -> Unit,
    navigateToDeviceIpInfo: () -> Unit,
    navigateToConnectOnDeviceOnStartUpInfo: () -> Unit,
) {
    Scaffold(
        modifier = modifier.fillMaxSize(),
        topBar = {
            MullvadSmallTopBar(
                title = stringResource(id = R.string.settings_vpn),
                navigationIcon = {
                    if (state.isModal()) {
                        NavigateCloseIconButton(onNavigateClose = onBackClick)
                    } else {
                        NavigateBackIconButton(onNavigateBack = onBackClick)
                    }
                },
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
                            state = state.value,
                            initialScrollToFeature = initialScrollToFeature,
                            navigateToAutoConnectScreen = navigateToAutoConnectScreen,
                            navigateToQuantumResistanceInfo = navigateToQuantumResistanceInfo,
                            navigateToLocalNetworkSharingInfo = navigateToLocalNetworkSharingInfo,
                            navigateToServerIpOverrides = navigateToServerIpOverrides,
                            navigateToAntiCensorship = navigateToAntiCensorship,
                            onToggleLocalNetworkSharing = onToggleLocalNetworkSharing,
                            navigateToMtuDialog = navigateToMtuDialog,
                            navigateToDns = navigateToDns,
                            onSelectQuantumResistanceSetting = onSelectQuantumResistanceSetting,
                            onToggleAutoStartAndConnectOnBoot = onToggleAutoStartAndConnectOnBoot,
                            onSelectDeviceIpVersion = onSelectDeviceIpVersion,
                            onToggleIpv6 = onToggleIpv6,
                            navigateToIpv6Info = navigateToIpv6Info,
                            navigateToDeviceIpInfo = navigateToDeviceIpInfo,
                            navigateToConnectOnDeviceOnStartUpInfo =
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
    navigateToAutoConnectScreen: () -> Unit,
    navigateToQuantumResistanceInfo: () -> Unit,
    navigateToLocalNetworkSharingInfo: () -> Unit,
    navigateToServerIpOverrides: () -> Unit,
    navigateToAntiCensorship: () -> Unit,
    onToggleLocalNetworkSharing: (Boolean) -> Unit,
    navigateToMtuDialog: (mtu: Mtu?) -> Unit,
    navigateToDns: () -> Unit,
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
            FeatureIndicator.CUSTOM_MTU -> VpnSettingItem.Mtu::class
            else -> null
        }?.let { clazz -> state.settings.indexOfFirstOrNull { it::class == clazz } } ?: 0

    val highlightAnimation = remember { Animatable(AlphaVisible) }
    if (initialScrollToFeature != null) {
        RunOnKeyChange(initialScrollToFeature) {
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
    val focusRequesters: Map<FeatureIndicator, FocusRequester> = remember {
        featureIndicators().associateWith { FocusRequester() }
    }
    if (initialScrollToFeature != null) {
        RunOnKeyChange(initialScrollToFeature) {
            focusRequesters[initialScrollToFeature]?.requestFocus()
        }
    }
    LazyColumn(
        modifier =
            Modifier.testTag(LAZY_LIST_VPN_SETTINGS_TEST_TAG)
                .fillMaxSize()
                .drawVerticalScrollbar(
                    state = lazyListState,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .padding(horizontal = Dimens.sideMarginNew),
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
                        ListItemInfo(
                            modifier = Modifier.padding(bottom = Dimens.largeSpacer).animateItem(),
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
                            modifier =
                                Modifier.animateItem()
                                    .focusRequester(
                                        focusRequesters.getValue(
                                            FeatureIndicator.QUANTUM_RESISTANCE
                                        )
                                    ),
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

                VpnSettingItem.DnsHeader ->
                    item(key = it::class.simpleName) {
                        NavigationListItem(
                            modifier =
                                Modifier.testTag(LAZY_LIST_DNS_SETTINGS_TEST_TAG).animateItem(),
                            title = stringResource(id = R.string.dns_settings),
                            onClick = navigateToDns,
                        )
                    }

                VpnSettingItem.Spacer ->
                    item(contentType = it::class.simpleName) {
                        Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing).animateItem())
                    }

                VpnSettingItem.SmallSpacer ->
                    item(contentType = it::class.simpleName) {
                        Spacer(modifier = Modifier.height(Dimens.tinyPadding).animateItem())
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
        VpnSettingsSideEffect.ShowToast.GenericError -> resources.getString(R.string.error_occurred)
    }

private fun Lc<Boolean, VpnSettingsUiState>.isModal() =
    when (this) {
        is Lc.Loading -> value
        is Lc.Content -> value.isModal
    }

// A list of feature indicators on this screen
private fun featureIndicators() =
    listOf(
        FeatureIndicator.LAN_SHARING,
        FeatureIndicator.QUANTUM_RESISTANCE,
        FeatureIndicator.CUSTOM_MTU,
    )
