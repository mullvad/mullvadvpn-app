package net.mullvad.mullvadvpn.compose.screen

import android.graphics.Bitmap
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.applist.AppData
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.HeaderSwitchComposeCell
import net.mullvad.mullvadvpn.compose.cell.SplitTunnelingCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.constant.SplitTunnelingContentKey
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.util.getApplicationIconBitmapOrNull
import net.mullvad.mullvadvpn.viewmodel.SplitTunnelingViewModel
import org.koin.androidx.compose.koinViewModel

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

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun SplitTunneling(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<SplitTunnelingViewModel>()
    val state by viewModel.uiState.collectAsState()
    val context = LocalContext.current
    val packageManager = remember(context) { context.packageManager }
    SplitTunnelingScreen(
        uiState = state,
        onShowSystemAppsClick = viewModel::onShowSystemAppsClick,
        onExcludeAppClick = viewModel::onExcludeAppClick,
        onIncludeAppClick = viewModel::onIncludeAppClick,
        onBackClick = navigator::navigateUp,
        onResolveIcon = { packageName ->
            packageManager.getApplicationIconBitmapOrNull(packageName)
        }
    )
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
    val focusManager = LocalFocusManager.current

    ScaffoldWithMediumTopBar(
        modifier = Modifier.fillMaxSize(),
        appBarTitle = stringResource(id = R.string.split_tunneling),
        navigationIcon = { NavigateBackIconButton(onBackClick) }
    ) { modifier, lazyListState ->
        LazyColumn(
            modifier = modifier,
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
                        MullvadCircularProgressIndicatorLarge()
                    }
                }
                is SplitTunnelingUiState.ShowAppList -> {
                    if (uiState.excludedApps.isNotEmpty()) {
                        itemWithDivider(
                            key = SplitTunnelingContentKey.EXCLUDED_APPLICATIONS,
                            contentType = ContentType.HEADER
                        ) {
                            HeaderCell(
                                modifier = Modifier.animateItemPlacement(),
                                text = stringResource(id = R.string.exclude_applications),
                                background = MaterialTheme.colorScheme.primary,
                            )
                        }
                        itemsIndexedWithDivider(
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
                        HeaderCell(
                            modifier = Modifier.animateItemPlacement(),
                            text = stringResource(id = R.string.all_applications),
                            background = MaterialTheme.colorScheme.primary,
                        )
                    }
                    itemsIndexedWithDivider(
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
