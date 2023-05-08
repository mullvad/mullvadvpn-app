package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.CircularProgressIndicator
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.sp
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
import net.mullvad.mullvadvpn.compose.theme.MullvadBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadDarkBlue
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite
import net.mullvad.mullvadvpn.compose.theme.MullvadWhite60

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
    val mediumPadding = dimensionResource(id = R.dimen.medium_padding)
    val progressSize = dimensionResource(id = R.dimen.progress_size)

    CollapsableAwareToolbarScaffold(
        backgroundColor = MullvadDarkBlue,
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
                    color = MullvadWhite60,
                    text = stringResource(id = R.string.split_tunneling_description),
                    fontSize = 13.sp,
                    modifier =
                        Modifier.padding(
                            start = mediumPadding,
                            end = mediumPadding,
                            bottom = mediumPadding
                        )
                )
            }
            when (uiState) {
                SplitTunnelingUiState.Loading -> {
                    item {
                        CircularProgressIndicator(
                            color = MullvadWhite,
                            modifier = Modifier.size(width = progressSize, height = progressSize)
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
                                        color = Color.White,
                                        fontSize = 18.sp,
                                        fontWeight = FontWeight.SemiBold
                                    )
                                },
                                bodyView = {},
                                background = MullvadBlue,
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
                        item { Spacer(modifier = Modifier.height(mediumPadding)) }
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
                                    color = Color.White,
                                    fontSize = 18.sp,
                                    fontWeight = FontWeight.SemiBold
                                )
                            },
                            bodyView = {},
                            background = MullvadBlue,
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
