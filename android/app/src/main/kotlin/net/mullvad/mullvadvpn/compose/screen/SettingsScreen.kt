package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ApiAccessListDestination
import com.ramcosta.composedestinations.generated.destinations.AppInfoDestination
import com.ramcosta.composedestinations.generated.destinations.DaitaDestination
import com.ramcosta.composedestinations.generated.destinations.MultihopDestination
import com.ramcosta.composedestinations.generated.destinations.NotificationSettingsDestination
import com.ramcosta.composedestinations.generated.destinations.ReportProblemDestination
import com.ramcosta.composedestinations.generated.destinations.SplitTunnelingDestination
import com.ramcosta.composedestinations.generated.destinations.VpnSettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createUriHook
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.SettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExternalLinkListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.tag.DAITA_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.MULTIHOP_CELL_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.VPN_SETTINGS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.util.Lc
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
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
@Destination<RootGraph>(style = TopLevelTransition::class)
@Composable
fun Settings(navigator: DestinationsNavigator) {
    val vm = koinViewModel<SettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    SettingsScreen(
        state = state,
        onVpnSettingCellClick = dropUnlessResumed { navigator.navigate(VpnSettingsDestination()) },
        onSplitTunnelingCellClick =
            dropUnlessResumed { navigator.navigate(SplitTunnelingDestination()) },
        onAppInfoClick = dropUnlessResumed { navigator.navigate(AppInfoDestination) },
        onApiAccessClick = dropUnlessResumed { navigator.navigate(ApiAccessListDestination) },
        onReportProblemCellClick =
            dropUnlessResumed { navigator.navigate(ReportProblemDestination) },
        onMultihopClick = dropUnlessResumed { navigator.navigate(MultihopDestination()) },
        onDaitaClick = dropUnlessResumed { navigator.navigate(DaitaDestination()) },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onNotificationSettingsCellClick =
            dropUnlessResumed { navigator.navigate(NotificationSettingsDestination()) },
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
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
    ) { modifier, lazyListState ->
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
            title = stringResource(id = R.string.settings_notifications),
            onClick = onNotificationSettingsCellClick,
            position = Position.Top,
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
