package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import android.net.Uri
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
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
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ApiAccessListDestination
import com.ramcosta.composedestinations.generated.destinations.ReportProblemDestination
import com.ramcosta.composedestinations.generated.destinations.SplitTunnelingDestination
import com.ramcosta.composedestinations.generated.destinations.VpnSettingsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.DefaultExternalLinkView
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.NavigationCellBody
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackDownIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.transitions.SettingsTransition
import net.mullvad.mullvadvpn.lib.common.util.openLink
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.DarkThemeState
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
                    isSupportedVersion = true,
                    isPlayBuild = false,
                    isMaterialYouTheme = false,
                    darkThemeState = DarkThemeState.SYSTEM
                ),
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(style = SettingsTransition::class)
@Composable
fun Settings(navigator: DestinationsNavigator) {
    val vm = koinViewModel<SettingsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    SettingsScreen(
        state = state,
        onVpnSettingCellClick = dropUnlessResumed { navigator.navigate(VpnSettingsDestination) },
        onSplitTunnelingCellClick =
            dropUnlessResumed { navigator.navigate(SplitTunnelingDestination) },
        onApiAccessClick = dropUnlessResumed { navigator.navigate(ApiAccessListDestination) },
        onReportProblemCellClick =
            dropUnlessResumed { navigator.navigate(ReportProblemDestination) },
        onUseMaterialYouThemeClick = vm::setUseMaterialYouTheme,
        onDarkThemeStateSelected = vm::onDarkThemeStateSelected,
        onBackClick = dropUnlessResumed { navigator.navigateUp() }
    )
}

@ExperimentalMaterial3Api
@Composable
fun SettingsScreen(
    state: SettingsUiState,
    onVpnSettingCellClick: () -> Unit = {},
    onSplitTunnelingCellClick: () -> Unit = {},
    onReportProblemCellClick: () -> Unit = {},
    onApiAccessClick: () -> Unit = {},
    onUseMaterialYouThemeClick: (Boolean) -> Unit = {},
    onDarkThemeStateSelected: (DarkThemeState) -> Unit = {},
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
            item {
                MaterialYou(state = state, onUseMaterialYouThemeClick = onUseMaterialYouThemeClick)
            }
            item { Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding)) }
            item { DarkTheme(state = state, onDarkThemeStateSelected = onDarkThemeStateSelected) }
            if (state.isLoggedIn) {
                item { Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding)) }
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

            item {
                NavigationComposeCell(
                    title = stringResource(id = R.string.settings_api_access),
                    onClick = onApiAccessClick
                )
            }
            item { Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing)) }

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
        showWarning = !state.isSupportedVersion,
        isRowEnabled = !state.isPlayBuild
    )

    if (!state.isSupportedVersion) {
        Text(
            text = stringResource(id = R.string.unsupported_version_description),
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
        bodyView =
            @Composable {
                DefaultExternalLinkView(
                    chevronContentDescription = faqGuideLabel,
                    colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onSecondary)
                )
            },
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
        bodyView =
            @Composable {
                DefaultExternalLinkView(
                    privacyPolicyLabel,
                    ColorFilter.tint(MaterialTheme.colorScheme.onSecondary)
                )
            },
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

@Composable
private fun MaterialYou(state: SettingsUiState, onUseMaterialYouThemeClick: (Boolean) -> Unit) {
    HeaderSwitchComposeCell(
        title = stringResource(id = R.string.use_material_you),
        isToggled = state.isMaterialYouTheme,
        onCellClicked = onUseMaterialYouThemeClick
    )
}

@Composable
private fun DarkTheme(
    state: SettingsUiState,
    onDarkThemeStateSelected: (DarkThemeState) -> Unit = {}
) {
    Column {
        HeaderCell(
            text = stringResource(id = R.string.use_dark_theme),
        )
        SelectableCell(
            title = stringResource(id = R.string.use_system_setting),
            isSelected = state.darkThemeState == DarkThemeState.SYSTEM,
            onCellClicked = { onDarkThemeStateSelected(DarkThemeState.SYSTEM) }
        )
        SelectableCell(
            title = stringResource(id = R.string.on),
            isSelected = state.darkThemeState == DarkThemeState.ON,
            onCellClicked = { onDarkThemeStateSelected(DarkThemeState.ON) }
        )
        SelectableCell(
            title = stringResource(id = R.string.off),
            isSelected = state.darkThemeState == DarkThemeState.OFF,
            onCellClicked = { onDarkThemeStateSelected(DarkThemeState.OFF) }
        )
    }
}
