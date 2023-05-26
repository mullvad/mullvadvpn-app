package net.mullvad.mullvadvpn.compose.screen

import android.net.Uri
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material.Text
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.DefaultExternalLinkView
import net.mullvad.mullvadvpn.compose.cell.NavigationCellBody
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.ui.extension.openLink

@OptIn(ExperimentalMaterialApi::class)
@Preview
@Composable
private fun PreviewSettings() {
    SettingsScreen(
        uiState =
            SettingsUiState(appVersion = "2222.2.2", isLoggedIn = true, isUpdateAvailable = true)
    )
}

@ExperimentalMaterialApi
@Composable
fun SettingsScreen(
    uiState: SettingsUiState,
    onVpnSettingCellClick: () -> Unit = {},
    onSplitTunnelingCellClick: () -> Unit = {},
    onReportProblemCellClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val context = LocalContext.current
    val lazyListState = rememberLazyListState()
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress
    val cellVerticalSpacing = Dimens.cellLabelVerticalPadding
    val cellHorizontalSpacing = Dimens.cellStartPadding
    val topPadding = 6.dp

    CollapsableAwareToolbarScaffold(
        backgroundColor = MaterialTheme.colorScheme.background,
        modifier = Modifier.fillMaxSize(),
        state = state,
        scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
        isEnabledWhenCollapsable = true,
        toolbar = {
            val scaffoldModifier =
                Modifier.road(
                    whenCollapsed = Alignment.TopCenter,
                    whenExpanded = Alignment.BottomStart
                )
            CollapsingTopBar(
                backgroundColor = MaterialTheme.colorScheme.secondary,
                onBackClicked = { onBackClick() },
                title = stringResource(id = R.string.settings),
                progress = progress,
                modifier = scaffoldModifier,
                backTitle = String(),
                shouldRotateBackButtonDown = true
            )
        },
    ) {
        LazyColumn(
            modifier =
                Modifier.drawVerticalScrollbar(lazyListState)
                    .testTag(LAZY_LIST_TEST_TAG)
                    .fillMaxWidth()
                    .wrapContentHeight()
                    .animateContentSize(),
            state = lazyListState
        ) {
            item { Spacer(modifier = Modifier.height(cellVerticalSpacing)) }
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
                            Uri.parse(context.resources.getString(R.string.download_url))
                        )
                    },
                    bodyView =
                        @Composable {
                            NavigationCellBody(
                                content = uiState.appVersion,
                                contentBodyDescription = stringResource(id = R.string.app_version),
                                isExternalLink = true,
                            )
                        },
                    showWarning = uiState.isUpdateAvailable,
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
                                    start = cellHorizontalSpacing,
                                    top = topPadding,
                                    end = cellHorizontalSpacing,
                                    bottom = cellVerticalSpacing,
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

            itemWithDivider {
                val privacyPolicyLabel = stringResource(id = R.string.privacy_policy_label)
                NavigationComposeCell(
                    title = privacyPolicyLabel,
                    bodyView = @Composable { DefaultExternalLinkView(privacyPolicyLabel) },
                    onClick = {
                        context.openLink(
                            Uri.parse(context.resources.getString(R.string.privacy_policy_url))
                        )
                    }
                )
            }
        }
    }
}
