package net.mullvad.mullvadvpn.compose.screen

import android.graphics.Bitmap
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.cell.BaseCell
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.SplitTunnelingCell
import net.mullvad.mullvadvpn.compose.component.CollapsingToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.constant.SplitTunnelingContentKey
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewSplitTunnelingScreen() {
    AppTheme {
        SplitTunnelingScreen(
            uiState =
                SplitTunnelingUiState.ShowAppList(
                    excludedApps =
                        listOf(
                            AppData(
                                packageName = "my.package.a",
                                name = "TitleA",
                                iconRes = R.drawable.icon_alert,
                            ),
                            AppData(
                                packageName = "my.package.b",
                                name = "TitleB",
                                iconRes = R.drawable.icon_chevron,
                            )
                        ),
                    includedApps =
                        listOf(
                            AppData(
                                packageName = "my.package.c",
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
    onShowSystemAppsClick: (show: Boolean) -> Unit = {},
    onExcludeAppClick: (packageName: String) -> Unit = {},
    onIncludeAppClick: (packageName: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onResolveIcon: (String) -> Bitmap? = { null },
) {
    val state = rememberCollapsingToolbarScaffoldState()
    val progress = state.toolbarState.progress
    val lazyListState = rememberLazyListState()
    val focusManager = LocalFocusManager.current

    CollapsingToolbarScaffold(
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
                modifier = scaffoldModifier
            )
        },
    ) {
        Surface(color = MaterialTheme.colorScheme.background) {
            LazyColumn(
                modifier = Modifier.drawVerticalScrollbar(state = lazyListState).fillMaxWidth(),
                horizontalAlignment = Alignment.CenterHorizontally,
                state = lazyListState
            ) {
                item(key = CommonContentKey.DESCRIPTION, contentType = ContentType.DESCRIPTION) {
                    Box(modifier = Modifier.fillMaxWidth()) {
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
                }
                when (uiState) {
                    SplitTunnelingUiState.Loading -> {
                        item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
                            CircularProgressIndicator(
                                color = MaterialTheme.colorScheme.onBackground,
                                modifier = Modifier.size(Dimens.progressIndicatorSize),
                                strokeCap = StrokeCap.Round
                            )
                        }
                    }
                    is SplitTunnelingUiState.ShowAppList -> {
                        if (uiState.excludedApps.isNotEmpty()) {
                            itemWithDivider(
                                key = SplitTunnelingContentKey.EXCLUDED_APPLICATIONS,
                                contentType = ContentType.HEADER
                            ) {
                                BaseCell(
                                    title = {
                                        Text(
                                            text =
                                                stringResource(id = R.string.exclude_applications),
                                            style = MaterialTheme.typography.titleMedium,
                                            color = MaterialTheme.colorScheme.onPrimary
                                        )
                                    },
                                    bodyView = {},
                                    background = MaterialTheme.colorScheme.primary,
                                )
                            }
                            itemsIndexed(
                                items = uiState.excludedApps,
                                key = { _, listItem -> listItem.packageName },
                                contentType = { _, _ -> ContentType.ITEM }
                            ) { index, listItem ->
                                SplitTunnelingCell(
                                    title = listItem.name,
                                    packageName = listItem.packageName,
                                    isSelected = true,
                                    modifier = Modifier.animateItemPlacement().fillMaxWidth(),
                                    onResolveIcon = onResolveIcon
                                ) {
                                    // Move focus down unless the clicked item was the last in this
                                    // section.
                                    if (index < uiState.excludedApps.size - 1) {
                                        focusManager.moveFocus(FocusDirection.Down)
                                    } else {
                                        focusManager.moveFocus(FocusDirection.Up)
                                    }

                                    onIncludeAppClick(listItem.packageName)
                                }
                            }
                            item(key = CommonContentKey.SPACER, contentType = ContentType.SPACER) {
                                Spacer(
                                    modifier =
                                        Modifier.animateItemPlacement().height(Dimens.mediumPadding)
                                )
                            }
                        }

                        itemWithDivider(
                            key = SplitTunnelingContentKey.SHOW_SYSTEM_APPLICATIONS,
                            contentType = ContentType.OTHER_ITEM
                        ) {
                            HeaderSwitchComposeCell(
                                title = stringResource(id = R.string.show_system_apps),
                                isToggled = uiState.showSystemApps,
                                onCellClicked = { newValue -> onShowSystemAppsClick(newValue) },
                                modifier = Modifier.animateItemPlacement()
                            )
                        }
                        itemWithDivider(
                            key = SplitTunnelingContentKey.INCLUDED_APPLICATIONS,
                            contentType = ContentType.HEADER
                        ) {
                            BaseCell(
                                modifier = Modifier.animateItemPlacement(),
                                title = {
                                    Text(
                                        text = stringResource(id = R.string.all_applications),
                                        style = MaterialTheme.typography.titleMedium,
                                        color = MaterialTheme.colorScheme.onPrimary
                                    )
                                },
                                bodyView = {},
                                background = MaterialTheme.colorScheme.primary,
                            )
                        }
                        itemsIndexed(
                            items = uiState.includedApps,
                            key = { _, listItem -> listItem.packageName },
                            contentType = { _, _ -> ContentType.ITEM }
                        ) { index, listItem ->
                            SplitTunnelingCell(
                                title = listItem.name,
                                packageName = listItem.packageName,
                                isSelected = false,
                                modifier = Modifier.animateItemPlacement().fillMaxWidth(),
                                onResolveIcon = onResolveIcon
                            ) {
                                // Move focus down unless the clicked item was the last in this
                                // section.
                                if (index < uiState.includedApps.size - 1) {
                                    focusManager.moveFocus(FocusDirection.Down)
                                } else {
                                    focusManager.moveFocus(FocusDirection.Up)
                                }

                                onExcludeAppClick(listItem.packageName)
                            }
                        }
                    }
                }
            }
        }
    }
}
