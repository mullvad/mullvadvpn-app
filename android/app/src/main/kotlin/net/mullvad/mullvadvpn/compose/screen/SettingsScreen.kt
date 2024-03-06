package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import android.net.Uri
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.DefaultExternalLinkView
import net.mullvad.mullvadvpn.compose.cell.NavigationCellBody
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackDownIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.destinations.ReportProblemDestination
import net.mullvad.mullvadvpn.compose.destinations.SplitTunnelingDestination
import net.mullvad.mullvadvpn.compose.destinations.VpnSettingsDestination
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SettingsTransition
import net.mullvad.mullvadvpn.lib.common.util.openLink
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.SettingsViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewSettings() {
    AppTheme {
        SettingsScreen(
            state =
                SettingsUiState(
                    appVersion = "2222.22",
                    isLoggedIn = true,
                    isUpdateAvailable = true,
                    isPlayBuild = false
                ),
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination(style = SettingsTransition::class)
@Composable
fun Settings(navigator: DestinationsNavigator) {
    val vm = koinViewModel<SettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    SettingsScreen(
        state = state,
        onVpnSettingCellClick = {
            navigator.navigate(VpnSettingsDestination) { launchSingleTop = true }
        },
        onSplitTunnelingCellClick = {
            navigator.navigate(SplitTunnelingDestination) { launchSingleTop = true }
        },
        onReportProblemCellClick = {
            navigator.navigate(ReportProblemDestination) { launchSingleTop = true }
        },
        onBackClick = navigator::navigateUp
    )
}

@ExperimentalMaterial3Api
@Composable
fun SettingsScreen(
    state: SettingsUiState,
    onVpnSettingCellClick: () -> Unit = {},
    onSplitTunnelingCellClick: () -> Unit = {},
    onReportProblemCellClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val context = LocalContext.current

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings),
        navigationIcon = { NavigateBackDownIconButton(onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_TEST_TAG).animateContentSize(),
            state = lazyListState
        ) {
            item { Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding)) }
            if (state.isLoggedIn) {
                item {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.settings_vpn),
                        onClick = onVpnSettingCellClick
                    )
                }
                item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
                item { SplitTunneling(onSplitTunnelingCellClick) }
                item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }
            }

            item { AppVersion(context, state) }

            item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

            itemWithDivider { ReportProblem(onReportProblemCellClick) }

            if (!state.isPlayBuild) {
                itemWithDivider { FaqAndGuides(context) }
            }

            itemWithDivider { PrivacyPolicy(context, state) }
        }
    }
}

@Composable
private fun SplitTunneling(onSplitTunnelingCellClick: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(id = R.string.split_tunneling),
        onClick = onSplitTunnelingCellClick
    )
}

@Composable
private fun AppVersion(context: Context, state: SettingsUiState) {
    NavigationComposeCell(
        title = stringResource(id = R.string.app_version),
        onClick = {
            context.openLink(
                Uri.parse(
                    context.resources
                        .getString(R.string.download_url)
                        .appendHideNavOnPlayBuild(state.isPlayBuild)
                )
            )
        },
        bodyView =
            @Composable {
                if (!state.isPlayBuild) {
                    NavigationCellBody(
                        content = state.appVersion,
                        contentBodyDescription = stringResource(id = R.string.app_version),
                        isExternalLink = true,
                    )
                } else {
                    Text(
                        text = state.appVersion,
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSecondary
                    )
                }
            },
        showWarning = state.isUpdateAvailable,
        isRowEnabled = !state.isPlayBuild
    )

    if (state.isUpdateAvailable) {
        Text(
            text = stringResource(id = R.string.update_available_footer),
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSecondary,
            modifier =
                Modifier.background(MaterialTheme.colorScheme.secondary)
                    .padding(
                        start = Dimens.cellStartPadding,
                        top = Dimens.cellTopPadding,
                        end = Dimens.cellStartPadding,
                        bottom = Dimens.cellLabelVerticalPadding,
                    )
        )
    }
}

@Composable
private fun ReportProblem(onReportProblemCellClick: () -> Unit) {
    NavigationComposeCell(
        title = stringResource(id = R.string.report_a_problem),
        onClick = { onReportProblemCellClick() }
    )
}

@Composable
private fun FaqAndGuides(context: Context) {
    val faqGuideLabel = stringResource(id = R.string.faqs_and_guides)
    NavigationComposeCell(
        title = faqGuideLabel,
        bodyView = @Composable { DefaultExternalLinkView(faqGuideLabel) },
        onClick = {
            context.openLink(Uri.parse(context.resources.getString(R.string.faqs_and_guides_url)))
        }
    )
}

@Composable
private fun PrivacyPolicy(context: Context, state: SettingsUiState) {
    val privacyPolicyLabel = stringResource(id = R.string.privacy_policy_label)
    NavigationComposeCell(
        title = privacyPolicyLabel,
        bodyView = @Composable { DefaultExternalLinkView(privacyPolicyLabel) },
        onClick = {
            context.openLink(
                Uri.parse(
                    context.resources
                        .getString(R.string.privacy_policy_url)
                        .appendHideNavOnPlayBuild(state.isPlayBuild)
                )
            )
        }
    )
}
