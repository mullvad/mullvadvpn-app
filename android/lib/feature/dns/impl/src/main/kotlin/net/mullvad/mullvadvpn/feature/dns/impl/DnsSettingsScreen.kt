package net.mullvad.mullvadvpn.feature.dns.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.animation.core.Animatable
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Add
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.android.awaitFrame
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavKey
import net.mullvad.mullvadvpn.feature.dns.api.CustomDnsNavResult
import net.mullvad.mullvadvpn.feature.dns.api.DnsSettingsNavKey
import net.mullvad.mullvadvpn.feature.dns.api.MalwareInfoNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.RunOnKeyChange
import net.mullvad.mullvadvpn.lib.common.compose.SETTINGS_HIGHLIGHT_REPEAT_COUNT
import net.mullvad.mullvadvpn.lib.common.compose.clickableAnnotatedString
import net.mullvad.mullvadvpn.lib.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.lib.common.compose.isTv
import net.mullvad.mullvadvpn.lib.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.lib.common.compose.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.util.openNetworkSettings
import net.mullvad.mullvadvpn.lib.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.ui.component.Accordion
import net.mullvad.mullvadvpn.lib.ui.component.SPACE_CHAR
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.DnsListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.InfoListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ListItemInfo
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_DNS_ADD_ITEM_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.CUSTOM_DNS_ITEM_X_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_DNS_SETTINGS_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaInvisible
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaVisible
import net.mullvad.mullvadvpn.lib.ui.util.applyIfNotNull
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview
@Composable
private fun PreviewDnsSettingsScreen() {
    AppTheme {
        DnsSettingsScreen(
            modifier = Modifier,
            state = Lc.Loading(Unit),
            snackbarHostState = SnackbarHostState(),
            selectedFeatureIndicator = null,
            navigateToDns = { _, _ -> },
            onToggleDnsClick = {},
            onToggleAllContentBlockers = {},
            onToggleBlockAds = {},
            onToggleBlockAdultContent = {},
            onToggleBlockGambling = {},
            onToggleBlockMalware = {},
            onToggleBlockSocialMedia = {},
            onToggleBlockTrackers = {},
            navigateToMalwareInfo = {},
            navigateToNetworkSettings = {},
            onBackClick = {},
        )
    }
}

@Composable
fun SharedTransitionScope.DnsSettings(
    navigator: Navigator,
    navArgs: DnsSettingsNavKey,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val vm = koinViewModel<DnsSettingsViewModel> { parametersOf(navArgs.isModal) }
    val snackbarHostState = remember { SnackbarHostState() }

    LocalResultStore.current.consumeResult<CustomDnsNavResult> { result ->
        when (result) {
            is CustomDnsNavResult.Success -> {
                if (!result.isDnsListEmpty) {
                    vm.onCustomDnsDialogSuccess()
                }
            }
            CustomDnsNavResult.Error -> vm.showGenericErrorToast()
        }
    }

    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            DnsSettingsSideEffect.NavigateToDnsDialog -> navigator.navigate(CustomDnsNavKey())
            DnsSettingsSideEffect.ShowToast.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.error_occurred)
                    )
                }
        }
    }

    val context = LocalContext.current
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
        selectedFeatureIndicator = navArgs.selectedFeature,
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
        navigateToMalwareInfo = dropUnlessResumed { navigator.navigate(MalwareInfoNavKey) },
        navigateToNetworkSettings = dropUnlessResumed { context.openNetworkSettings() },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Suppress("LongParameterList")
@Composable
fun DnsSettingsScreen(
    modifier: Modifier,
    state: Lc<Unit, DnsSettingsUiState>,
    snackbarHostState: SnackbarHostState,
    selectedFeatureIndicator: FeatureIndicator?,
    navigateToDns: (index: Int?, address: String?) -> Unit,
    onToggleDnsClick: (Boolean) -> Unit,
    onToggleAllContentBlockers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToNetworkSettings: () -> Unit,
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
        Box(modifier = modifier) {
            when (state) {
                is Lc.Loading -> Loading()
                is Lc.Content ->
                    Content(
                        state = state.value,
                        selectedFeatureIndicator = selectedFeatureIndicator,
                        navigateToDns = navigateToDns,
                        onToggleDnsClick = onToggleDnsClick,
                        onToggleAllBlockers = onToggleAllContentBlockers,
                        onToggleBlockAds = onToggleBlockAds,
                        onToggleBlockAdultContent = onToggleBlockAdultContent,
                        onToggleBlockGambling = onToggleBlockGambling,
                        onToggleBlockMalware = onToggleBlockMalware,
                        onToggleBlockSocialMedia = onToggleBlockSocialMedia,
                        onToggleBlockTrackers = onToggleBlockTrackers,
                        navigateToMalwareInfo = navigateToMalwareInfo,
                        navigateToNetworkSettings = navigateToNetworkSettings,
                    )
            }
        }
    }
}

@Suppress("LongMethod", "CyclomaticComplexMethod")
@Composable
private fun Content(
    state: DnsSettingsUiState,
    selectedFeatureIndicator: FeatureIndicator?,
    navigateToDns: (index: Int?, address: String?) -> Unit,
    onToggleDnsClick: (Boolean) -> Unit,
    onToggleAllBlockers: (Boolean) -> Unit,
    onToggleBlockAds: (Boolean) -> Unit,
    onToggleBlockAdultContent: (Boolean) -> Unit,
    onToggleBlockGambling: (Boolean) -> Unit,
    onToggleBlockMalware: (Boolean) -> Unit,
    onToggleBlockSocialMedia: (Boolean) -> Unit,
    onToggleBlockTrackers: (Boolean) -> Unit,
    navigateToMalwareInfo: () -> Unit,
    navigateToNetworkSettings: () -> Unit,
) {
    val highlightAnimation = remember { Animatable(AlphaVisible) }
    if (selectedFeatureIndicator != null) {
        RunOnKeyChange(selectedFeatureIndicator) {
            repeat(times = SETTINGS_HIGHLIGHT_REPEAT_COUNT) {
                highlightAnimation.animateTo(AlphaInvisible)
                highlightAnimation.animateTo(AlphaVisible)
            }
        }
    }

    @Composable
    fun highlightBackgroundAlpha(featureIndicator: FeatureIndicator): Float =
        if (selectedFeatureIndicator == featureIndicator) {
            highlightAnimation.value
        } else {
            1.0f
        }

    val lazyListState =
        rememberLazyListState(
            initialFirstVisibleItemIndex =
                when (selectedFeatureIndicator) {
                    FeatureIndicator.DNS_CONTENT_BLOCKERS -> CONTENT_BLOCKERS_INDEX
                    FeatureIndicator.CUSTOM_DNS -> 0 // Scroll to bottom
                    else -> 0
                }
        )

    LaunchedEffect(state.customDnsEnabled) {
        if (state.customDnsEnabled) {
            // This added so that the list state has time to update the totalItemsCount before
            // scrolling to the last item.
            // It is not guaranteed that the item count is correct after this, but theoretically it
            // should help.
            awaitFrame()

            // Attempt to scroll to the last item in the list, if the list is not laid out yet
            // (i.e. totalItemsCount = 0), then we will just let the user enter the screen at the
            // top.
            val lastIndex = lazyListState.layoutInfo.totalItemsCount - 1
            if (lastIndex >= 0) {
                lazyListState.requestScrollToItem(lastIndex)
            }
        }
    }

    val focusCustomDnsRequester = remember { FocusRequester() }
    val focusDnsBlockersRequester = remember { FocusRequester() }
    if (selectedFeatureIndicator != null) {
        RunOnKeyChange(selectedFeatureIndicator) {
            if (selectedFeatureIndicator == FeatureIndicator.CUSTOM_DNS) {
                focusCustomDnsRequester.requestFocus()
            } else if (selectedFeatureIndicator == FeatureIndicator.DNS_CONTENT_BLOCKERS) {
                focusDnsBlockersRequester.requestFocus()
            }
        }
    }
    LazyColumn(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            Modifier.drawVerticalScrollbar(
                    state = lazyListState,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .testTag(LAZY_LIST_DNS_SETTINGS_TEST_TAG)
                .padding(horizontal = Dimens.sideMarginNew),
        state = lazyListState,
    ) {
        item(key = ContentKey.IMAGE) {
            // Scale image to fit width up to certain width
            Image(
                contentScale = ContentScale.FillWidth,
                modifier =
                    Modifier.animateItem()
                        .widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                        .fillMaxWidth(),
                painter = painterResource(id = R.drawable.dns_content_blockers_illustration),
                contentDescription = stringResource(R.string.dns_content_blockers),
            )
        }

        item(key = ContentKey.DESCRIPTION) { Description() }

        item(key = ContentKey.EXTRA_INFORMATION) {
            if (!isTv()) {
                var expandedState by rememberSaveable { mutableStateOf(false) }

                Accordion(
                    modifier = Modifier.padding(bottom = Dimens.cellVerticalSpacing),
                    title = stringResource(R.string.if_you_have_any_issues),
                    expandedText =
                        clickableAnnotatedString(
                            text =
                                buildString {
                                    appendLine(
                                        stringResource(R.string.dns_settings_issues_first_paragraph)
                                    )
                                    append(
                                        stringResource(
                                            R.string.dns_settings_issues_second_paragraph
                                        )
                                    )
                                },
                            argument = stringResource(R.string.system_network_settings),
                            linkStyle =
                                SpanStyle(
                                    color = MaterialTheme.colorScheme.onSurface,
                                    textDecoration = TextDecoration.Underline,
                                ),
                            onClick = { navigateToNetworkSettings() },
                        ),
                    icon = Icons.Rounded.Info,
                    iconContentDescription = stringResource(R.string.info),
                    isExpanded = expandedState,
                    onClick = { expandedState = !expandedState },
                )
            }
        }

        contentBlockers(
            focusDnsBlockersRequester = focusDnsBlockersRequester,
            highlightBackgroundAlpha = { highlightBackgroundAlpha(it) },
            numberOfBlockersEnabled = state.defaultDnsOptions.numberOfBlockersEnabled(),
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

        itemWithDivider(key = ContentKey.ENABLE_CUSTOM_DNS) {
            SwitchListItem(
                modifier = Modifier.animateItem().focusRequester(focusCustomDnsRequester),
                position = if (state.customDnsEnabled) Position.Top else Position.Single,
                title = stringResource(R.string.enable_custom_dns),
                isToggled = state.customDnsEnabled,
                isEnabled = !state.defaultDnsOptions.isAnyBlockerEnabled,
                onCellClicked = { newValue -> onToggleDnsClick(newValue) },
                backgroundAlpha = highlightBackgroundAlpha(FeatureIndicator.CUSTOM_DNS),
            )
        }

        if (state.customDnsEnabled) {
            itemsIndexedWithDivider(
                items = state.customDnsEntries,
                key = { _, item -> item.address },
            ) { index, item ->
                DnsListItem(
                    modifier =
                        Modifier.animateItem().testTag(CUSTOM_DNS_ITEM_X_TEST_TAG.format(index)),
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
                item(key = ContentKey.CUSTOM_DNS_ADD) {
                    MullvadListItem(
                        modifier = Modifier.animateItem().testTag(CUSTOM_DNS_ADD_ITEM_TEST_TAG),
                        hierarchy = Hierarchy.Child1,
                        position = Position.Bottom,
                        onClick = { navigateToDns(null, null) },
                        content = { Text(text = stringResource(id = R.string.add_a_server)) },
                        trailingContent = {
                            Icon(imageVector = Icons.Rounded.Add, contentDescription = null)
                        },
                    )
                }
            }
        }

        if (state.defaultDnsOptions.isAnyBlockerEnabled) {
            item(key = ContentKey.CUSTOM_DNS_DISABLE_INFO) {
                ListItemInfo(
                    modifier = Modifier.animateItem(),
                    text =
                        stringResource(
                            id = R.string.custom_dns_disable_mode_subtitle,
                            stringResource(id = R.string.dns_content_blockers),
                        ),
                )
            }
        }

        item(key = ContentKey.SPACER) {
            Spacer(modifier = Modifier.animateItem().height(Dimens.cellVerticalSpacing))
        }
    }
}

@Composable
private fun LazyItemScope.Description() {
    ScreenDescription(
        modifier = Modifier.animateItem().padding(top = Dimens.smallPadding),
        text =
            buildAnnotatedString {
                appendLine(annotatedStringResource(R.string.dns_settings_description_title))
                appendLine(annotatedStringResource(R.string.dns_settings_description_first_item))
                appendLine()
                appendLine(annotatedStringResource(R.string.dns_settings_description_second_item))
                if (isTv()) {
                    appendLine()
                    appendLine(annotatedStringResource(R.string.dns_settings_description_warning))
                }
            },
    )
}

@Composable
private fun LazyItemScope.ContentBlockersHeader(
    highlightBackgroundAlpha: @Composable (FeatureIndicator) -> Float,
    numberOfBlockersEnabled: Int,
) {
    InfoListItem(
        modifier = Modifier.animateItem(),
        position = Position.Top,
        content = {
            Row {
                Text(
                    text = stringResource(R.string.dns_content_blockers),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (numberOfBlockersEnabled > 0) {
                    Text(SPACE_CHAR.toString())
                    Text(
                        stringResource(R.string.number_parentheses, numberOfBlockersEnabled),
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                }
            }
        },
        backgroundAlpha = highlightBackgroundAlpha(FeatureIndicator.DNS_CONTENT_BLOCKERS),
    )
}

@Suppress("LongMethod")
private fun LazyListScope.contentBlockers(
    focusDnsBlockersRequester: FocusRequester,
    highlightBackgroundAlpha: @Composable (FeatureIndicator) -> Float,
    numberOfBlockersEnabled: Int,
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
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKERS_HEADER) {
        ContentBlockersHeader(
            numberOfBlockersEnabled = numberOfBlockersEnabled,
            highlightBackgroundAlpha = highlightBackgroundAlpha,
        )
    }

    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_ALL) {
        ContentBlocker(
            focusRequester = focusDnsBlockersRequester,
            title = stringResource(R.string.all),
            isToggled = defaultDnsOptions.isAllBlockersEnabled,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleAllBlockers,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_ADS) {
        ContentBlocker(
            title = stringResource(R.string.block_ads_title),
            isToggled = defaultDnsOptions.blockAds,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockAds,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_TRACKERS) {
        ContentBlocker(
            title = stringResource(R.string.block_trackers_title),
            isToggled = defaultDnsOptions.blockTrackers,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockTrackers,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_MALWARE) {
        ContentBlocker(
            title = stringResource(R.string.block_malware_title),
            isToggled = defaultDnsOptions.blockMalware,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockMalware,
            onInfoClicked = navigateToMalwareInfo,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_GAMBLING) {
        ContentBlocker(
            title = stringResource(R.string.block_gambling_title),
            isToggled = defaultDnsOptions.blockGambling,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockGambling,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_ADULT_CONTENT) {
        ContentBlocker(
            title = stringResource(R.string.block_adult_content_title),
            isToggled = defaultDnsOptions.blockAdultContent,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockAdultContent,
        )
    }
    itemWithDivider(key = ContentKey.DNS_CONTENT_BLOCKER_SOCIAL_MEDIA) {
        ContentBlocker(
            title = stringResource(R.string.block_social_media_title),
            isToggled = defaultDnsOptions.blockSocialMedia,
            isEnabled = contentBlockersEnabled,
            onClicked = onToggleBlockSocialMedia,
            position = Position.Bottom,
        )
    }

    if (!contentBlockersEnabled) {
        item(key = ContentKey.DNS_CONTENT_BLOCKERS_DISABLE_INFO) {
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
        item(key = ContentKey.CONTENT_BLOCKERS_SPACER) {
            Spacer(modifier = Modifier.height(Dimens.mediumPadding).animateItem())
        }
    }
}

@Composable
private fun LazyItemScope.ContentBlocker(
    focusRequester: FocusRequester = FocusRequester(),
    title: String,
    isToggled: Boolean,
    isEnabled: Boolean,
    position: Position = Position.Middle,
    onClicked: (Boolean) -> Unit,
    onInfoClicked: (() -> Unit)? = null,
) {
    SwitchListItem(
        modifier = Modifier.animateItem().focusRequester(focusRequester),
        position = position,
        hierarchy = Hierarchy.Child1,
        title = title,
        isToggled = isToggled,
        isEnabled = isEnabled,
        onCellClicked = { onClicked(it) },
        onInfoClicked = onInfoClicked,
    )
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}

private object ContentKey {
    const val IMAGE = "image"
    const val DESCRIPTION = "description"
    const val EXTRA_INFORMATION = "extra_information"
    const val DNS_CONTENT_BLOCKERS_HEADER = "dns_content_blockers_header"
    const val DNS_CONTENT_BLOCKER_ALL = "dns_content_blocker_all"
    const val DNS_CONTENT_BLOCKER_ADS = "dns_content_blocker_ads"
    const val DNS_CONTENT_BLOCKER_TRACKERS = "dns_content_blocker_trackers"
    const val DNS_CONTENT_BLOCKER_MALWARE = "dns_content_blocker_malware"
    const val DNS_CONTENT_BLOCKER_GAMBLING = "dns_content_blocker_gambling"
    const val DNS_CONTENT_BLOCKER_ADULT_CONTENT = "dns_content_blocker_adult_content"
    const val DNS_CONTENT_BLOCKER_SOCIAL_MEDIA = "dns_content_blocker_social_media"
    const val DNS_CONTENT_BLOCKERS_DISABLE_INFO = "dns_content_blockers_disable_info"
    const val CONTENT_BLOCKERS_SPACER = "content_blockers_spacer"
    const val ENABLE_CUSTOM_DNS = "enable_custom_dns"
    const val CUSTOM_DNS_ADD = "custom_dns_add"
    const val CUSTOM_DNS_DISABLE_INFO = "custom_dns_disable_info"
    const val SPACER = "spacer"
}

private const val CONTENT_BLOCKERS_INDEX = 2
