package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.SplitTunnelingCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeCell
import net.mullvad.mullvadvpn.compose.component.CollapsableAwareToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.compose.theme.AppTheme
import net.mullvad.mullvadvpn.compose.theme.Dimens
import net.mullvad.mullvadvpn.compose.util.ContentKey
import net.mullvad.mullvadvpn.compose.util.ContentType

@Preview
@Composable
fun PreviewSplitTunnelingScreen() {
    AppTheme {
        SplitTunnelingScreen(
            uiState =
                SplitTunnelingUiState.ShowAppList(
                    excludedApps =
                        listOf(
                            AppData(
                                packageName = "Package C",
                                name = "TitleA",
                                iconRes = R.drawable.icon_alert
                            ),
                            AppData(
                                packageName = "Package G",
                                name = "TitleB",
                                iconRes = R.drawable.icon_chevron,
                            )
                        ),
                    includedApps =
                        listOf(
                            AppData(
                                packageName = "Package I",
                                name = "TitleC",
                                iconRes = R.drawable.icon_alert
                            )
                        ),
                    showSystemApps = true
                )
        )
    }
}

@Composable
@OptIn(ExperimentalFoundationApi::class)
fun SplitTunnelingScreen(
    uiState: SplitTunnelingUiState = SplitTunnelingUiState.Loading,
    onShowSystemAppsClicked: (show: Boolean) -> Unit = {},
    onExcludeAppClick: (packageName: String) -> Unit = {},
    onIncludeAppClick: (packageName: String) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress
    val lazyListState = rememberLazyListState()

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
                backgroundColor = MaterialTheme.colorScheme.background,
                onBackClicked = { onBackClick() },
                title = stringResource(id = R.string.split_tunneling),
                progress = progress,
                modifier = scaffoldModifier,
                backTitle = stringResource(id = R.string.settings)
            )
        },
    ) {
        LazyColumn(
            modifier = Modifier.drawVerticalScrollbar(state = lazyListState),
            horizontalAlignment = Alignment.CenterHorizontally,
            state = lazyListState
        ) {
            item(key = ContentKey.DESCRIPTION, contentType = ContentType.DESCRIPTION) {
                Text(
                    style = MaterialTheme.typography.labelMedium,
                    text = stringResource(id = R.string.split_tunneling_description),
                    modifier =
                        Modifier.padding(
                            start = Dimens.mediumPadding,
                            end = Dimens.mediumPadding,
                            bottom = Dimens.mediumPadding
                        )
                )
            }
            when (uiState) {
                SplitTunnelingUiState.Loading -> {
                    item(key = ContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
                        CircularProgressIndicator(
                            color = MaterialTheme.colorScheme.onBackground,
                            modifier =
                                Modifier.size(
                                    width = Dimens.progressIndicatorSize,
                                    height = Dimens.progressIndicatorSize
                                )
                        )
                    }
                }
                is SplitTunnelingUiState.ShowAppList -> {
                    if (uiState.excludedApps.isNotEmpty()) {
                        itemWithDivider(
                            key = ContentKey.EXCLUDED_APPLICATIONS,
                            contentType = ContentType.HEADER
                        ) {
                            BaseCell(
                                title = {
                                    Text(
                                        text = stringResource(id = R.string.exclude_applications),
                                        style = MaterialTheme.typography.titleMedium
                                    )
                                },
                                bodyView = {},
                                background = MaterialTheme.colorScheme.primary,
                            )
                        }
                        items(
                            items = uiState.excludedApps,
                            key = { listItem -> listItem.packageName },
                            contentType = { ContentType.ITEM }
                        ) { listItem ->
                            SplitTunnelingCell(
                                title = listItem.name,
                                packageName = listItem.packageName,
                                isSelected = true,
                                modifier = Modifier.animateItemPlacement().fillMaxWidth()
                            ) {
                                onIncludeAppClick(listItem.packageName)
                            }
                        }
                        item(key = ContentKey.SPACER, contentType = ContentType.SPACER) {
                            Spacer(modifier = Modifier.height(Dimens.mediumPadding))
                        }
                    }

                    itemWithDivider(
                        key = ContentKey.SHOW_SYSTEM_APPLICATIONS,
                        contentType = ContentType.OTHER_ITEM
                    ) {
                        SwitchComposeCell(
                            title = stringResource(id = R.string.show_system_apps),
                            isToggled = uiState.showSystemApps,
                            onCellClicked = { newValue -> onShowSystemAppsClicked(newValue) }
                        )
                    }
                    itemWithDivider(
                        key = ContentKey.INCLUDED_APPLICATIONS,
                        contentType = ContentType.HEADER
                    ) {
                        BaseCell(
                            title = {
                                Text(
                                    text = stringResource(id = R.string.all_applications),
                                    style = MaterialTheme.typography.titleMedium
                                )
                            },
                            bodyView = {},
                            background = MaterialTheme.colorScheme.primary,
                        )
                    }
                    items(
                        items = uiState.includedApps,
                        key = { listItem -> listItem.packageName },
                        contentType = { ContentType.ITEM }
                    ) { listItem ->
                        SplitTunnelingCell(
                            title = listItem.name,
                            packageName = listItem.packageName,
                            isSelected = false,
                            modifier = Modifier.animateItemPlacement().fillMaxWidth()
                        ) {
                            onExcludeAppClick(listItem.packageName)
                        }
                    }
                }
            }
        }
    }
}
