package net.mullvad.mullvadvpn.compose.screen

import android.net.Uri
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.ExperimentalMaterialApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.CellSubtitle
import net.mullvad.mullvadvpn.compose.cell.ExternalLinkCellBody
import net.mullvad.mullvadvpn.compose.cell.ExternalLinkComposeCell
import net.mullvad.mullvadvpn.compose.cell.NavigationCellBody
import net.mullvad.mullvadvpn.compose.cell.NavigationComposeCell
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.getExpiryQuantityString
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SettingsUiState
import net.mullvad.mullvadvpn.compose.test.LAZY_LIST_TEST_TAG
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.dimensions.defaultDimensions
import org.joda.time.DateTime
import org.joda.time.Duration

@OptIn(ExperimentalMaterialApi::class)
@Preview
@Composable
private fun PreviewSettings() {
    SettingsScreen(uiState = SettingsUiState(true, DateTime.parse("2011-11-11"), "2222.22", true))
}

@ExperimentalMaterialApi
@Composable
fun SettingsScreen(
    uiState: SettingsUiState,
    onAccountCellClick: () -> Unit = {},
    onVpnSettingCellClick: () -> Unit = {},
    onSplitTunnelingCellClick: () -> Unit = {},
    onReportProblemCellClick: () -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val lazyListState = rememberLazyListState()
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress
    var remainingTimeText by remember { mutableStateOf("") }

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
                backgroundColor = MullvadDarkBlue,
                onBackClicked = { onBackClick() },
                title = stringResource(id = R.string.settings),
                progress = progress,
                modifier = scaffoldModifier,
                backTitle = String(),
                shouldRotateBackButton = true
            )
        },
    ) {
        remainingTimeText =
            LocalContext.current.resources.getExpiryQuantityString(
                uiState.accountExpiry ?: DateTime.now()
            )
        LazyColumn(
            modifier =
                Modifier.drawVerticalScrollbar(lazyListState)
                    .testTag(LAZY_LIST_TEST_TAG)
                    .fillMaxWidth()
                    .wrapContentHeight()
                    .animateContentSize(),
            state = lazyListState
        ) {
            if (uiState.isLoggedIn) {
                itemWithDivider {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.settings_account),
                        bodyView =
                            @Composable {
                                NavigationCellBody(
                                    title = stringResource(id = R.string.settings_account),
                                    content = remainingTimeText,
                                    isTypeOfContentError =
                                        Duration(DateTime.now(), uiState.accountExpiry)
                                            .isShorterThan(Duration.ZERO)
                                )
                            },
                        onClick = { onAccountCellClick() }
                    )
                }
                itemWithDivider {
                    NavigationComposeCell(
                        title = stringResource(id = R.string.settings_vpn),
                        onClick = { onVpnSettingCellClick() }
                    )
                }

                item {
                    Spacer(modifier = Modifier.height(defaultDimensions.cellVerticalSpacing))
                    NavigationComposeCell(
                        title = stringResource(id = R.string.split_tunneling),
                        onClick = { onSplitTunnelingCellClick() }
                    )
                }
            }
            itemWithDivider {
                Spacer(modifier = Modifier.height(defaultDimensions.cellVerticalSpacing))
                ExternalLinkComposeCell(
                    title = stringResource(id = R.string.app_version),
                    uri = Uri.parse(stringResource(id = R.string.download_url)),
                    bodyView =
                        @Composable {
                            ExternalLinkCellBody(
                                title = stringResource(id = R.string.app_version),
                                content = uiState.appVersion
                            )
                        },
                    showWarning = uiState.updateAvailable
                )
            }
            if (uiState.updateAvailable) {
                item {
                    CellSubtitle(content = stringResource(id = R.string.update_available_footer))
                }
            }

            itemWithDivider {
                Spacer(modifier = Modifier.height(defaultDimensions.cellVerticalSpacing))
                NavigationComposeCell(
                    title = stringResource(id = R.string.report_a_problem),
                    onClick = { onReportProblemCellClick() }
                )
            }

            itemWithDivider {
                ExternalLinkComposeCell(
                    title = stringResource(id = R.string.faqs_and_guides),
                    uri = Uri.parse(stringResource(id = R.string.faqs_and_guides_url))
                )
            }

            itemWithDivider {
                ExternalLinkComposeCell(
                    title = stringResource(id = R.string.privacy_policy_label),
                    uri = Uri.parse(stringResource(id = R.string.privacy_policy_url))
                )
            }
        }
    }
}
