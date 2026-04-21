package net.mullvad.mullvadvpn.feature.dns.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.ContentBlockersInfoNavKey
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsInfoNavKey
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavKey
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavResult
import net.mullvad.mullvadvpn.feature.dns.api.DnsSettingsNavKey
import net.mullvad.mullvadvpn.feature.dns.api.MalwareInfoNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.ui.component.DividerButton
import net.mullvad.mullvadvpn.lib.ui.component.SPACE_CHAR
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.DnsListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExpandableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ListItemInfo
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewDnsSettingsScreen() {
    DnsSettingsScreen(
        modifier = Modifier,
        state = Lc.Loading(Unit),
        snackbarHostState = SnackbarHostState(),
        navigateToCustomDnsInfo = {},
        navigateToDns = { _, _ -> },
        onToggleDnsClick = {},
        onToggleAllContentBlockers = {},
        onToggleBlockAds = {},
        onToggleBlockAdultContent = {},
        onToggleBlockGambling = {},
        onToggleBlockMalware = {},
        onToggleBlockSocialMedia = {},
        onToggleBlockTrackers = {},
        onToggleContentBlockersExpanded = {},
        navigateToContentBlockersInfo = {},
        navigateToMalwareInfo = {},
        onBackClick = {},
    )
}

@Composable
fun SharedTransitionScope.DnsSettings(
    navigator: Navigator,
    navArgs: DnsSettingsNavKey,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val resultStore = LocalResultStore.current
    val vm = koinViewModel<DnsSettingsViewModel> { parametersOf(navArgs.isModal) }
    val snackbarHostState = remember { SnackbarHostState() }

    resultStore.consumeResult<CustomDnsNavResult> { result ->
        when (result) {
            is CustomDnsNavResult.Success -> {
                vm.showApplySettingChangesWarningToast()
            }
            CustomDnsNavResult.Error -> {
                vm.showGenericErrorToast()
            }
        }
    }

    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            DnsSettingsSideEffect.NavigateToDnsDialog -> navigator.navigate(CustomDnsNavKey())
            DnsSettingsSideEffect.ShowToast.ApplySettingWarning ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message =
                            resources.getString(R.string.settings_changes_effect_warning_short)
                    )
                }
            DnsSettingsSideEffect.ShowToast.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.error_occurred)
                    )
                }
        }
    }

    val state by vm.uiState.collectAsStateWithLifecycle()
    DnsSettingsScreen(
        modifier =
            Modifier.applyIfNotNull(navArgs.selectedFeature) {
                sharedBounds(
                    rememberSharedContentState(key = it),
                    animatedVisibilityScope = animatedVisibilityScope,
                )
            },
        state = state,
        snackbarHostState = snackbarHostState,
        navigateToCustomDnsInfo = dropUnlessResumed { navigator.navigate(CustomDnsInfoNavKey) },
        navigateToDns =
            dropUnlessResumed { index: Int?, address: String? ->
                navigator.navigate(CustomDnsNavKey(index, address))
            },
        onToggleDnsClick = vm::onToggleCustomDns,
        onToggleAllContentBlockers = vm::onToggleAllBlockers,
        onToggleBlockAds = vm::onToggleBlockAds,
        onToggleBlockAdultContent = vm::onToggleBlockAdultContent,
        onToggleBlockGambling = vm::onToggleBlockGambling,
        onToggleBlockMalware = vm::onToggleBlockMalware,
        onToggleBlockSocialMedia = vm::onToggleBlockSocialMedia,
        onToggleBlockTrackers = vm::onToggleBlockTrackers,
        onToggleContentBlockersExpanded = vm::onToggleContentBlockersExpanded,
        navigateToContentBlockersInfo =
            dropUnlessResumed { navigator.navigate(ContentBlockersInfoNavKey) },
        navigateToMalwareInfo = dropUnlessResumed { navigator.navigate(MalwareInfoNavKey) },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Suppress("LongParameterList")
@Composable
fun DnsSettingsScreen(
    modifier: Modifier,
    state: Lc<Unit, DnsSettingsUiState>,
    snackbarHostState: SnackbarHostState,
    navigateToCustomDnsInfo: () -> Unit,
    navigateToDns: (index: Int?, address: String?) -> Unit,
    onToggleDnsClick: (Boolean) -> Unit,
    onToggleAllContentBlockers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    onToggleContentBlockersExpanded: () -> Unit,
    navigateToContentBlockersInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithSmallTopBar(
        modifier = modifier,
        appBarTitle = stringResource(id = R.string.dns_settings),
        snackbarHostState = snackbarHostState,
        navigationIcon = {
            if (state.contentOrNull()?.isModal == true) {
                NavigateCloseIconButton(onNavigateClose = onBackClick)
            } else {
                NavigateBackIconButton(onNavigateBack = onBackClick)
            }
        },
    ) { modifier ->
        val lazyListState = rememberLazyListState()
        LazyColumn(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                modifier
                    .drawVerticalScrollbar(
                        state = lazyListState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .testTag(LAZY_LIST_TEST_TAG)
                    .padding(horizontal = Dimens.sideMarginNew)
                    .animateContentSize(),
            state = lazyListState,
        ) {
            when (state) {
                is Lc.Loading -> loading()
                is Lc.Content ->
                    content(
                        state = state.value,
                        navigateToDns = navigateToDns,
                        onToggleDnsClick = onToggleDnsClick,
                        onToggleAllBlockers = onToggleAllContentBlockers,
                        onToggleBlockAds = onToggleBlockAds,
                        onToggleBlockAdultContent = onToggleBlockAdultContent,
                        onToggleBlockGambling = onToggleBlockGambling,
                        onToggleBlockMalware = onToggleBlockMalware,
                        onToggleBlockSocialMedia = onToggleBlockSocialMedia,
                        onToggleBlockTrackers = onToggleBlockTrackers,
                        onToggleContentBlockersExpanded = onToggleContentBlockersExpanded,
                        navigateToContentBlockersInfo = navigateToContentBlockersInfo,
                        navigateToMalwareInfo = navigateToMalwareInfo,
                        navigateToCustomDnsInfo = navigateToCustomDnsInfo,
                    )
            }
        }
    }
}

@Suppress("LongMethod")
private fun LazyListScope.content(
    state: DnsSettingsUiState,
    navigateToDns: (index: Int?, address: String?) -> Unit,
    onToggleDnsClick: (Boolean) -> Unit,
    onToggleAllBlockers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    onToggleContentBlockersExpanded: () -> Unit,
    navigateToContentBlockersInfo: () -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToCustomDnsInfo: () -> Unit,
) {
    item {
        ContentBlockersHeader(
            contentBlockersExpanded = state.contentBlockersExpanded,
            numberOfBlockersEnabled = state.defaultDnsOptions.numberOfBlockersEnabled(),
            navigateToContentBlockersInfo = navigateToContentBlockersInfo,
            onToggleContentBlockersExpanded = onToggleContentBlockersExpanded,
        )
    }

    if (state.contentBlockersExpanded) {
        contentBlockers(
            contentBlockersEnabled = state.contentBlockersEnabled,
            defaultDnsOptions = state.defaultDnsOptions,
            onToggleAllBlockers = onToggleAllBlockers,
            onToggleBlockAds = onToggleBlockAds,
            onToggleBlockTrackers = onToggleBlockTrackers,
            onToggleBlockMalware = onToggleBlockMalware,
            onToggleBlockAdultContent = onToggleBlockAdultContent,
            onToggleBlockGambling = onToggleBlockGambling,
            onToggleBlockSocialMedia = onToggleBlockSocialMedia,
            navigateToMalwareInfo = navigateToMalwareInfo,
        )
    }

    if (!state.contentBlockersEnabled) {
        item {
            ListItemInfo(
                text =
                    stringResource(
                        id = R.string.dns_content_blockers_subtitle,
                        stringResource(id = R.string.enable_custom_dns),
                    ),
                modifier = Modifier.animateItem(),
            )
        }
    } else {
        item { Spacer(modifier = Modifier.height(Dimens.tinyPadding).animateItem()) }
    }

    item {
        SwitchListItem(
            modifier = Modifier.animateItem(),
            position = if (state.customDnsEnabled) Position.Top else Position.Single,
            title = stringResource(R.string.enable_custom_dns),
            isToggled = state.customDnsEnabled,
            isEnabled = !state.defaultDnsOptions.isAnyBlockerEnabled,
            onCellClicked = { newValue -> onToggleDnsClick(newValue) },
            onInfoClicked = { navigateToCustomDnsInfo() },
        )
    }

    if (state.customDnsEnabled) {
        itemsIndexedWithDivider(
            items = state.customDnsEntries,
            key = { _, item -> item.address },
        ) { index, item ->
            DnsListItem(
                modifier = Modifier.animateItem(),
                hierarchy = Hierarchy.Child1,
                position = Position.Middle,
                address = item.address,
                isUnreachableLocalDnsWarningVisible =
                    item.isLocal && state.showUnreachableLocalDnsWarning,
                isUnreachableIpv6DnsWarningVisible =
                    item.isIpv6 && state.showUnreachableIpv6DnsWarning,
                onClick = { navigateToDns(index, item.address) },
            )
        }

        if (state.customDnsEntries.isNotEmpty()) {
            item {
                MullvadListItem(
                    modifier = Modifier.animateItem(),
                    hierarchy = Hierarchy.Child1,
                    position = Position.Bottom,
                    onClick = { navigateToDns(null, null) },
                    content = { Text(text = stringResource(id = R.string.add_a_server)) },
                    trailingContent = {
                        DividerButton(
                            onClick = { navigateToDns(null, null) },
                            icon = Icons.Rounded.Add,
                        )
                    },
                )
            }
        }
    }

    if (state.defaultDnsOptions.isAnyBlockerEnabled) {
        item {
            ListItemInfo(
                modifier = Modifier.animateItem(),
                text =
                    stringResource(
                        id = R.string.custom_dns_disable_mode_subtitle,
                        stringResource(id = R.string.dns_content_blockers),
                    ),
            )
        }
    } else if (state.customDnsEntries.isEmpty()) {
        item {
            ListItemInfo(
                modifier = Modifier.animateItem(),
                text = stringResource(id = R.string.custom_dns_footer),
            )
        }
    }
}

@Composable
private fun LazyItemScope.ContentBlockersHeader(
    contentBlockersExpanded: Boolean,
    numberOfBlockersEnabled: Int,
    navigateToContentBlockersInfo: () -> Unit,
    onToggleContentBlockersExpanded: () -> Unit,
) {
    ExpandableListItem(
        modifier = Modifier.animateItem(),
        position = if (contentBlockersExpanded) Position.Top else Position.Single,
        content = { _ ->
            Row {
                Text(
                    modifier = Modifier.weight(1f),
                    text = stringResource(R.string.dns_content_blockers),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (numberOfBlockersEnabled > 0) {
                    Text(SPACE_CHAR.toString())
                    Text(
                        stringResource(R.string.number_parentheses, numberOfBlockersEnabled),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        },
        isExpanded = contentBlockersExpanded,
        onInfoClicked = navigateToContentBlockersInfo,
        onCellClicked = { onToggleContentBlockersExpanded() },
    )
}

private fun LazyListScope.contentBlockers(
    contentBlockersEnabled: Boolean,
    defaultDnsOptions: DefaultDnsOptions,
    onToggleAllBlockers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    navigateToMalwareInfo: () -> Unit,
) {
    item {
        ContentBlocker(
            title = stringResource(R.string.all),
            isToggled = defaultDnsOptions.isAllBlockersEnabled,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleAllBlockers,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_ads_title),
            isToggled = defaultDnsOptions.blockAds,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockAds,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_trackers_title),
            isToggled = defaultDnsOptions.blockTrackers,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockTrackers,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_malware_title),
            isToggled = defaultDnsOptions.blockMalware,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockMalware,
            onInfoClicked = navigateToMalwareInfo,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_gambling_title),
            isToggled = defaultDnsOptions.blockGambling,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockGambling,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_adult_content_title),
            isToggled = defaultDnsOptions.blockAdultContent,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockAdultContent,
        )
    }
    item {
        ContentBlocker(
            title = stringResource(R.string.block_social_media_title),
            isToggled = defaultDnsOptions.blockSocialMedia,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockSocialMedia,
            position = Position.Bottom,
        )
    }
}

@Composable
private fun LazyItemScope.ContentBlocker(
    title: String,
    isToggled: Boolean,
    isEnabled: Boolean,
    position: Position = Position.Middle,
    onClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    SwitchListItem(
        modifier = Modifier.animateItem(),
        position = position,
        hierarchy = Hierarchy.Child1,
        title = title,
        isToggled = isToggled,
        isEnabled = isEnabled,
        onCellClicked = { onClicked(it) },
        onInfoClicked = onInfoClicked,
    )
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
