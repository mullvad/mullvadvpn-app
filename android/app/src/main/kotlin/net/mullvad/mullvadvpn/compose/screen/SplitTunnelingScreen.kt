package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.compose.theme.Dimens

@Preview
@Composable
fun PreviewSplitTunnelingScreen() {
    SplitTunnelingScreen(
        uiState =
            SplitTunnelingUiState.Data(
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

@Composable
fun SplitTunnelingScreen(
    uiState: SplitTunnelingUiState = SplitTunnelingUiState.Loading,
    onShowSystemAppsClicked: (show: Boolean) -> Unit = {},
    addToExcluded: (packageName: String) -> Unit = {},
    removeFromExcluded: (packageName: String) -> Unit = {},
    onBackClick: () -> Unit = {},
) {
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress

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
                backTitle = stringResource(id = R.string.settings_advanced)
            )
        },
    ) {
        LazyColumn(horizontalAlignment = Alignment.CenterHorizontally) {
            item(contentType = TYPE_DESCRIPTION) {
                Text(
                    style = MaterialTheme.typography.bodySmall,
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
                    item {
                        CircularProgressIndicator(
                            color = MaterialTheme.colorScheme.onBackground,
                            modifier =
                                Modifier.size(
                                    width = Dimens.progressSize,
                                    height = Dimens.progressSize
                                )
                        )
                    }
                }
                is SplitTunnelingUiState.Data -> {
                    if (uiState.excludedApps.isNotEmpty()) {
                        itemWithDivider(contentType = TYPE_TITLE) {
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
                            contentType = { TYPE_APPLICATION }
                        ) { listItem ->
                            SplitTunnelingCell(
                                title = listItem.name,
                                packageName = listItem.packageName,
                                isSelected = true
                            ) {
                                removeFromExcluded(listItem.packageName)
                            }
                        }
                        item { Spacer(modifier = Modifier.height(Dimens.mediumPadding)) }
                    }

                    itemWithDivider(contentType = TYPE_SWITCH_CELL) {
                        SwitchComposeCell(
                            title = stringResource(id = R.string.show_system_apps),
                            isToggled = uiState.showSystemApps,
                            onCellClicked = { newValue -> onShowSystemAppsClicked(newValue) }
                        )
                    }
                    itemWithDivider(contentType = TYPE_TITLE) {
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
                        contentType = { TYPE_APPLICATION }
                    ) { listItem ->
                        SplitTunnelingCell(
                            title = listItem.name,
                            packageName = listItem.packageName,
                            isSelected = false
                        ) {
                            addToExcluded(listItem.packageName)
                        }
                    }
                }
            }
        }
    }
}

const val TYPE_DESCRIPTION = 1
const val TYPE_TITLE = 2
const val TYPE_SWITCH_CELL = 3
const val TYPE_APPLICATION = 4
