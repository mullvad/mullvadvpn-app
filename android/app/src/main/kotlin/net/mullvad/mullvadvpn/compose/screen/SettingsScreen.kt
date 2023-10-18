package net.mullvad.mullvadvpn.compose.screen

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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.DefaultExternalLinkView
import net.mullvad.mullvadvpn.compose.cell.NavigationCellBody
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.NavigateBackDownIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.constant.IS_PLAY_BUILD
import net.mullvad.mullvadvpn.lib.common.util.openLink
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewSettings() {
    AppTheme {
        SettingsScreen(
            uiState =
                SettingsUiState(
                    appVersion = "2222.22",
                    isLoggedIn = true,
                    isUpdateAvailable = true
                ),
            enterTransitionEndAction = MutableSharedFlow()
        )
    }
}

@ExperimentalMaterial3Api
@Composable
fun SettingsScreen(
    uiState: SettingsUiState,
    enterTransitionEndAction: SharedFlow<Unit>,
    onVpnSettingCellClick: () -> Unit = {},
    onSplitTunnelingCellClick: () -> Unit = {},
    onReportProblemCellClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val context = LocalContext.current
    val backgroundColor = MaterialTheme.colorScheme.background
    val systemUiController = rememberSystemUiController()

    LaunchedEffect(Unit) {
        systemUiController.setNavigationBarColor(backgroundColor)
        enterTransitionEndAction.collect { systemUiController.setStatusBarColor(backgroundColor) }
    }

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.settings),
        navigationIcon = { NavigateBackDownIconButton(onBackClick) },
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier.testTag(LAZY_LIST_TEST_TAG).animateContentSize(),
            state = lazyListState
        ) {
            item { Spacer(modifier = Modifier.height(Dimens.cellLabelVerticalPadding)) }
            if (uiState.isLoggedIn) {
                item {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.settings_vpn),
                        onClick = { onVpnSettingCellClick() }
                    )
                }

                item {
                    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
                    NavigationComposeCell(
                        title = stringResource(id = R.string.split_tunneling),
                        onClick = { onSplitTunnelingCellClick() }
                    )
                    Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
                }
            }
            item {
                NavigationComposeCell(
                    title = stringResource(id = R.string.app_version),
                    onClick = {
                        context.openLink(
                            Uri.parse(
                                context.resources
                                    .getString(R.string.download_url)
                                    .appendHideNavOnPlayBuild()
                            )
                        )
                    },
                    bodyView =
                        @Composable {
                            if (IS_PLAY_BUILD.not()) {
                                NavigationCellBody(
                                    content = uiState.appVersion,
                                    contentBodyDescription =
                                        stringResource(id = R.string.app_version),
                                    isExternalLink = true,
                                )
                            } else {
                                Text(
                                    text = uiState.appVersion,
                                    style = MaterialTheme.typography.labelMedium,
                                    color = MaterialTheme.colorScheme.onSecondary
                                )
                            }
                        },
                    showWarning = uiState.isUpdateAvailable,
                    isRowEnabled = IS_PLAY_BUILD.not()
                )
            }
            if (uiState.isUpdateAvailable) {
                item {
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

            itemWithDivider {
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing))
                NavigationComposeCell(
                    title = stringResource(id = R.string.report_a_problem),
                    onClick = { onReportProblemCellClick() }
                )
            }

            if (IS_PLAY_BUILD.not()) {
                itemWithDivider {
                    val faqGuideLabel = stringResource(id = R.string.faqs_and_guides)
                    NavigationComposeCell(
                        title = faqGuideLabel,
                        bodyView = @Composable { DefaultExternalLinkView(faqGuideLabel) },
                        onClick = {
                            context.openLink(
                                Uri.parse(context.resources.getString(R.string.faqs_and_guides_url))
                            )
                        }
                    )
                }
            }

            itemWithDivider {
                val privacyPolicyLabel = stringResource(id = R.string.privacy_policy_label)
                NavigationComposeCell(
                    title = privacyPolicyLabel,
                    bodyView = @Composable { DefaultExternalLinkView(privacyPolicyLabel) },
                    onClick = {
                        context.openLink(
                            Uri.parse(
                                context.resources
                                    .getString(R.string.privacy_policy_url)
                                    .appendHideNavOnPlayBuild()
                            )
                        )
                    }
                )
            }
        }
    }
}
