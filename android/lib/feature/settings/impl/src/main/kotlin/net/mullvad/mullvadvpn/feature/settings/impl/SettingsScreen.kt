package net.mullvad.mullvadvpn.feature.settings.impl

import android.os.Build
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.assureHasDetailPane
import net.mullvad.mullvadvpn.common.compose.createUriHook
import net.mullvad.mullvadvpn.common.compose.isTv
import net.mullvad.mullvadvpn.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.common.compose.navigateReplaceIfDetailPane
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.anticensorship.api.AntiCensorshipNavKey
import net.mullvad.mullvadvpn.feature.apiaccess.api.ApiAccessNavKey
import net.mullvad.mullvadvpn.feature.appearance.api.AppearanceNavKey
import net.mullvad.mullvadvpn.feature.appinfo.api.AppInfoNavKey
import net.mullvad.mullvadvpn.feature.autoconnect.api.AutoConnectNavKey
import net.mullvad.mullvadvpn.feature.daita.api.DaitaNavKey
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.feature.multihop.api.MultihopNavKey
import net.mullvad.mullvadvpn.feature.notification.api.NotificationSettingsNavKey
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNavKey
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.feature.splittunneling.api.SplitTunnelingNavKey
import net.mullvad.mullvadvpn.feature.vpnsettings.api.VpnSettingsNavKey
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.common.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.lib.ui.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExternalLinkListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.VPN_SETTINGS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|+")
@Composable
private fun PreviewSettingsScreen(
    @PreviewParameter(SettingsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, SettingsUiState>
) {
    AppTheme {
        SettingsScreen(
            state = state,
            onVpnSettingCellClick = {},
            onSplitTunnelingCellClick = {},
            onAppInfoClick = {},
            onReportProblemCellClick = {},
            onApiAccessClick = {},
            onMultihopClick = {},
            onDaitaClick = {},
            onBackClick = {},
            onNotificationSettingsCellClick = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun Settings(navigator: Navigator) {
    val vm = koinViewModel<SettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    val isTv = isTv()

    BackHandler(enabled = navigator.screenIsListDetailTargetWidth) {
        navigator.goBackUntil(SettingsNavKey, inclusive = true)
    }

    navigator.assureHasDetailPane<SettingsNavKey>(DaitaNavKey())

    SettingsScreen(
        state = state,
        onVpnSettingCellClick =
            dropUnlessResumed {
                if (navigator.screenIsListDetailTargetWidth) {
                    val detailKey = if (isTv) AntiCensorshipNavKey() else AutoConnectNavKey
                    navigator.navigate(VpnSettingsNavKey(), detailKey)
                } else {
                    navigator.navigate(VpnSettingsNavKey())
                }
            },
        onSplitTunnelingCellClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(SplitTunnelingNavKey()) },
        onAppInfoClick = dropUnlessResumed { navigator.navigateReplaceIfDetailPane(AppInfoNavKey) },
        onApiAccessClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(ApiAccessNavKey) },
        onReportProblemCellClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(ProblemReportNavKey) },
        onMultihopClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(MultihopNavKey()) },
        onDaitaClick = dropUnlessResumed { navigator.navigateReplaceIfDetailPane(DaitaNavKey()) },
        onNotificationSettingsCellClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(NotificationSettingsNavKey) },
        onAppObfuscationClick =
            dropUnlessResumed { navigator.navigateReplaceIfDetailPane(AppearanceNavKey) },
        onLanguageClick =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                dropUnlessResumed { navigator.navigateReplaceIfDetailPane(LanguageNavKey) }
            } else {
                null
            },
        onBackClick = dropUnlessResumed { navigator.goBackUntil(SettingsNavKey, inclusive = true) },
    )
}

@Composable
fun SettingsScreen(
    state: Lc<Unit, SettingsUiState>,
    onVpnSettingCellClick: () -> Unit,
    onSplitTunnelingCellClick: () -> Unit,
    onAppInfoClick: () -> Unit,
    onReportProblemCellClick: () -> Unit,
    onApiAccessClick: () -> Unit,
    onMultihopClick: () -> Unit,
    onDaitaClick: () -> Unit,
    onBackClick: () -> Unit,
    onNotificationSettingsCellClick: () -> Unit,
    onLanguageClick: (() -> Unit)? = null,
    onAppObfuscationClick: () -> Unit = {},
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
    ) { modifier, lazyListState: LazyListState ->
        LazyColumn(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                modifier
                    .testTag(LAZY_LIST_TEST_TAG)
                    .padding(horizontal = Dimens.sideMarginNew)
                    .animateContentSize(),
            state = lazyListState,
        ) {
            when (state) {
                is Lc.Loading -> loading()
                is Lc.Content -> {
                    content(
                        state = state.value,
                        onVpnSettingCellClick = onVpnSettingCellClick,
                        onSplitTunnelingCellClick = onSplitTunnelingCellClick,
                        onAppInfoClick = onAppInfoClick,
                        onReportProblemCellClick = onReportProblemCellClick,
                        onApiAccessClick = onApiAccessClick,
                        onMultihopClick = onMultihopClick,
                        onDaitaClick = onDaitaClick,
                        onNotificationSettingsCellClick = onNotificationSettingsCellClick,
                        onAppObfuscationClick = onAppObfuscationClick,
                        onLanguageClick = onLanguageClick,
                    )
                }
            }
        }
    }
}

private fun LazyListScope.content(
    state: SettingsUiState,
    onVpnSettingCellClick: () -> Unit,
    onSplitTunnelingCellClick: () -> Unit,
    onAppInfoClick: () -> Unit,
    onReportProblemCellClick: () -> Unit,
    onApiAccessClick: () -> Unit,
    onMultihopClick: () -> Unit,
    onDaitaClick: () -> Unit,
    onNotificationSettingsCellClick: () -> Unit,
    onAppObfuscationClick: () -> Unit = {},
    onLanguageClick: (() -> Unit)? = null,
) {
    if (state.isLoggedIn) {
        itemWithDivider {
            DaitaListItem(isDaitaEnabled = state.isDaitaEnabled, onDaitaClick = onDaitaClick)
        }
        itemWithDivider {
            MultihopCell(
                isMultihopEnabled = state.multihopEnabled,
                onMultihopClick = onMultihopClick,
            )
        }
        itemWithDivider {
            NavigationListItem(
                title = stringResource(id = R.string.settings_vpn),
                onClick = onVpnSettingCellClick,
                testTag = VPN_SETTINGS_CELL_TEST_TAG,
                position = Position.Bottom,
            )
        }
        item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
        item { SplitTunneling(onSplitTunnelingCellClick) }
        item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
    }

    item {
        NavigationListItem(
            title = stringResource(id = R.string.settings_api_access),
            onClick = onApiAccessClick,
        )
    }

    item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

    itemWithDivider {
        NavigationListItem(
            title = stringResource(id = R.string.appearance),
            onClick = onAppObfuscationClick,
            position = Position.Top,
        )
    }

    if (onLanguageClick != null) {
        itemWithDivider {
            NavigationListItem(
                title = stringResource(id = R.string.language),
                onClick = onLanguageClick,
                position = Position.Middle,
            )
        }
    }

    itemWithDivider {
        NavigationListItem(
            title = stringResource(id = R.string.settings_notifications),
            onClick = onNotificationSettingsCellClick,
            position = Position.Middle,
        )
    }

    item { AppInfo(onAppInfoClick, state) }

    item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

    itemWithDivider { ReportProblem(onReportProblemCellClick) }

    if (!state.isPlayBuild) {
        itemWithDivider { FaqAndGuides() }
    }

    itemWithDivider { PrivacyPolicy(state) }

    item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
}

@Composable
private fun SplitTunneling(onSplitTunnelingCellClick: () -> Unit) {
    NavigationListItem(
        title = stringResource(id = R.string.split_tunneling),
        onClick = onSplitTunnelingCellClick,
    )
}

@Composable
private fun AppInfo(navigateToAppInfo: () -> Unit, state: SettingsUiState) {
    NavigationListItem(
        title = stringResource(id = R.string.app_info),
        subtitle = state.appVersion,
        subTitleTextDirection = TextDirection.Ltr,
        showWarning = !state.isSupportedVersion,
        position = Position.Bottom,
        onClick = navigateToAppInfo,
    )
}

@Composable
private fun ReportProblem(onReportProblemCellClick: () -> Unit) {
    NavigationListItem(
        title = stringResource(id = R.string.report_a_problem),
        onClick = { onReportProblemCellClick() },
        position = Position.Top,
    )
}

@Composable
private fun FaqAndGuides() {
    val faqGuideLabel = stringResource(id = R.string.faqs_and_guides)
    val openFaqAndGuides =
        LocalUriHandler.current.createUriHook(stringResource(R.string.faqs_and_guides_url))

    ExternalLinkListItem(
        title = faqGuideLabel,
        onClick = openFaqAndGuides,
        position = Position.Middle,
    )
}

@Composable
private fun PrivacyPolicy(state: SettingsUiState) {
    val privacyPolicyLabel = stringResource(id = R.string.privacy_policy_label)

    val openPrivacyPolicy =
        LocalUriHandler.current.createUriHook(
            stringResource(R.string.privacy_policy_url).appendHideNavOnPlayBuild(state.isPlayBuild)
        )

    ExternalLinkListItem(
        title = privacyPolicyLabel,
        onClick = openPrivacyPolicy,
        position = Position.Bottom,
    )
}

@Composable
private fun DaitaListItem(isDaitaEnabled: Boolean, onDaitaClick: () -> Unit) {
    val title = stringResource(id = R.string.daita)
    NavigationListItem(
        title = title,
        subtitle =
            stringResource(
                if (isDaitaEnabled) {
                    R.string.on
                } else {
                    R.string.off
                }
            ),
        onClick = onDaitaClick,
        position = Position.Top,
        testTag = DAITA_CELL_TEST_TAG,
    )
}

@Composable
private fun MultihopCell(isMultihopEnabled: Boolean, onMultihopClick: () -> Unit) {
    val title = stringResource(id = R.string.multihop)
    NavigationListItem(
        title = title,
        subtitle =
            stringResource(
                if (isMultihopEnabled) {
                    R.string.on
                } else {
                    R.string.off
                }
            ),
        onClick = onMultihopClick,
        position = Position.Middle,
        testTag = MULTIHOP_CELL_TEST_TAG,
    )
}

private fun LazyListScope.loading() {
    item { MullvadCircularProgressIndicatorLarge() }
}
