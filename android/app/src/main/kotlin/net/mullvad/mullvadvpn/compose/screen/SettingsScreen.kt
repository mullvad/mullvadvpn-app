package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Error
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
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
import com.ramcosta.composedestinations.generated.destinations.ReportProblemDestination
import com.ramcosta.composedestinations.generated.destinations.SplitTunnelingDestination
import com.ramcosta.composedestinations.generated.destinations.VpnSettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.DefaultExternalLinkView
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.TwoRowCell
import net.mullvad.mullvadvpn.compose.component.NavigateCloseIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createUriHook
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.preview.SettingsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.DAITA_CELL_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.VPN_SETTINGS_CELL_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.TopLevelTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Supported|+")
@Composable
private fun PreviewSettingsScreen(
    @PreviewParameter(SettingsUiStatePreviewParameterProvider::class) state: SettingsUiState
) {
    AppTheme { SettingsScreen(state = state, {}, {}, {}, {}, {}, {}, {}, {}, {}) }
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
        onWidgetClick = dropUnlessResumed { /*navigator.navigate(WidgetDestination)*/ },
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
fun SettingsScreen(
    state: SettingsUiState,
    onVpnSettingCellClick: () -> Unit,
    onSplitTunnelingCellClick: () -> Unit,
    onAppInfoClick: () -> Unit,
    onReportProblemCellClick: () -> Unit,
    onApiAccessClick: () -> Unit,
    onMultihopClick: () -> Unit,
    onDaitaClick: () -> Unit,
    onWidgetClick: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings),
        navigationIcon = { NavigateCloseIconButton(onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_TEST_TAG).animateContentSize(),
            state = lazyListState,
        ) {
            if (state.isLoggedIn) {
                itemWithDivider {
                    DaitaCell(isDaitaEnabled = state.isDaitaEnabled, onDaitaClick = onDaitaClick)
                }
                itemWithDivider {
                    MultihopCell(
                        isMultihopEnabled = state.multihopEnabled,
                        onMultihopClick = onMultihopClick,
                    )
                }
                itemWithDivider {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.settings_vpn),
                        onClick = onVpnSettingCellClick,
                        testTag = VPN_SETTINGS_CELL_TEST_TAG,
                    )
                }
                item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
                item { SplitTunneling(onSplitTunnelingCellClick) }
                item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
                item { Widget(onWidgetClick) }
                item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
            }

            item {
                NavigationComposeCell(
                    title = stringResource(id = R.string.settings_api_access),
                    onClick = onApiAccessClick,
                )
            }
            item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

            item { AppInfo(onAppInfoClick, state) }

            item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

            itemWithDivider { ReportProblem(onReportProblemCellClick) }

            if (!state.isPlayBuild) {
                itemWithDivider { FaqAndGuides() }
            }

            itemWithDivider { PrivacyPolicy(state) }
        }
    }
}

@Composable
private fun SplitTunneling(onSplitTunnelingCellClick: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(id = R.string.split_tunneling),
        onClick = onSplitTunnelingCellClick,
    )
}

@Composable
private fun Widget(onWidgetClick: () -> Unit) {
    NavigationComposeCell(title = "Widget", onClick = onWidgetClick)
}

@Composable
private fun AppInfo(navigateToAppInfo: () -> Unit, state: SettingsUiState) {
    TwoRowCell(
        titleText = stringResource(id = R.string.app_info),
        subtitleText = state.appVersion,
        bodyView = {
            Row {
                if (!state.isSupportedVersion) {
                    Icon(
                        imageVector = Icons.Default.Error,
                        modifier = Modifier.padding(end = Dimens.smallPadding),
                        contentDescription = null,
                        tint = MaterialTheme.colorScheme.error,
                    )
                }
                Icon(
                    Icons.Default.ChevronRight,
                    contentDescription = stringResource(R.string.app_info),
                    tint = MaterialTheme.colorScheme.onPrimary,
                )
            }
        },
        onCellClicked = navigateToAppInfo,
    )
}

@Composable
private fun ReportProblem(onReportProblemCellClick: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(id = R.string.report_a_problem),
        onClick = { onReportProblemCellClick() },
    )
}

@Composable
private fun FaqAndGuides() {
    val faqGuideLabel = stringResource(id = R.string.faqs_and_guides)
    val openFaqAndGuides =
        LocalUriHandler.current.createUriHook(stringResource(R.string.faqs_and_guides_url))

    NavigationComposeCell(
        title = faqGuideLabel,
        bodyView = {
            DefaultExternalLinkView(
                chevronContentDescription = faqGuideLabel,
                tint = MaterialTheme.colorScheme.onPrimary,
            )
        },
        onClick = openFaqAndGuides,
    )
}

@Composable
private fun PrivacyPolicy(state: SettingsUiState) {
    val privacyPolicyLabel = stringResource(id = R.string.privacy_policy_label)

    val openPrivacyPolicy =
        LocalUriHandler.current.createUriHook(
            stringResource(R.string.privacy_policy_url).appendHideNavOnPlayBuild(state.isPlayBuild)
        )

    NavigationComposeCell(
        title = privacyPolicyLabel,
        bodyView = {
            DefaultExternalLinkView(
                chevronContentDescription = privacyPolicyLabel,
                tint = MaterialTheme.colorScheme.onPrimary,
            )
        },
        onClick = openPrivacyPolicy,
    )
}

@Composable
private fun DaitaCell(isDaitaEnabled: Boolean, onDaitaClick: () -> Unit) {
    val title = stringResource(id = R.string.daita)
    TwoRowCell(
        titleText = title,
        subtitleText =
            stringResource(
                if (isDaitaEnabled) {
                    R.string.on
                } else {
                    R.string.off
                }
            ),
        onCellClicked = onDaitaClick,
        bodyView = {
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = title,
                tint = MaterialTheme.colorScheme.onPrimary,
            )
        },
        modifier = Modifier.testTag(DAITA_CELL_TEST_TAG),
    )
}

@Composable
private fun MultihopCell(isMultihopEnabled: Boolean, onMultihopClick: () -> Unit) {
    val title = stringResource(id = R.string.multihop)
    TwoRowCell(
        titleText = title,
        subtitleText =
            stringResource(
                if (isMultihopEnabled) {
                    R.string.on
                } else {
                    R.string.off
                }
            ),
        onCellClicked = onMultihopClick,
        bodyView = {
            Icon(
                imageVector = Icons.Default.ChevronRight,
                contentDescription = title,
                tint = MaterialTheme.colorScheme.onPrimary,
            )
        },
    )
}
