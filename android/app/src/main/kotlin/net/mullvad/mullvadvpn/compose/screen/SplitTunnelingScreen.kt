package net.mullvad.mullvadvpn.compose.screen

import android.graphics.Bitmap
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.focus.FocusDirection
import androidx.compose.ui.focus.FocusManager
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
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.component.textResource
import net.mullvad.mullvadvpn.compose.constant.CommonContentKey
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.constant.SplitTunnelingContentKey
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.state.SplitTunnelingUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDisabled
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible
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
                    enabled = true,
                    excludedApps =
                        listOf(
                            AppData(
                                packageName = "my.package.a",
                                name = "TitleA",
                                iconRes = R.drawable.icon_alert
                            ),
                            AppData(
                                packageName = "my.package.b",
                                name = "TitleB",
                                iconRes = R.drawable.icon_chevron
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
        onShowSplitTunneling = viewModel::enableSplitTunneling,
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
fun SplitTunnelingScreen(
    uiState: SplitTunnelingUiState,
    onShowSplitTunneling: (Boolean) -> Unit = {},
    onShowSystemAppsClick: (show: Boolean) -> Unit = {},
    onExcludeAppClick: (packageName: String) -> Unit = {},
    onIncludeAppClick: (packageName: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onResolveIcon: (String) -> Bitmap? = { null }
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
            enabledToggle(enabled = uiState.enabled, onShowSplitTunneling)
            description(enabled = uiState.enabled)
            spacer(index = 0)
            when (uiState) {
                is SplitTunnelingUiState.Loading -> {
                    loading()
                }
                is SplitTunnelingUiState.ShowAppList -> {
                    appList(
                        uiState = uiState,
                        focusManager = focusManager,
                        onShowSystemAppsClick = onShowSystemAppsClick,
                        onExcludeAppClick = onExcludeAppClick,
                        onIncludeAppClick = onIncludeAppClick,
                        onResolveIcon = onResolveIcon
                    )
                }
            }
        }
    }
}

private fun LazyListScope.enabledToggle(enabled: Boolean, onShowSplitTunneling: (Boolean) -> Unit) {
    item {
        HeaderSwitchComposeCell(
            title = textResource(id = R.string.enable),
            isToggled = enabled,
            onCellClicked = onShowSplitTunneling
        )
    }
}

private fun LazyListScope.description(enabled: Boolean) {
    item(key = CommonContentKey.DESCRIPTION, contentType = ContentType.DESCRIPTION) {
        SwitchComposeSubtitleCell(
            text =
                if (enabled) {
                    stringResource(id = R.string.split_tunneling_description)
                } else {
                    stringResource(id = R.string.split_tunneling_disabled_description)
                }
        )
    }
}

private fun LazyListScope.loading() {
    item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge()
    }
}

private fun LazyListScope.appList(
    uiState: SplitTunnelingUiState.ShowAppList,
    focusManager: FocusManager,
    onShowSystemAppsClick: (show: Boolean) -> Unit,
    onExcludeAppClick: (packageName: String) -> Unit,
    onIncludeAppClick: (packageName: String) -> Unit,
    onResolveIcon: (String) -> Bitmap?
) {
    if (uiState.excludedApps.isNotEmpty()) {
        headerItem(
            key = SplitTunnelingContentKey.EXCLUDED_APPLICATIONS,
            textId = R.string.exclude_applications,
            enabled = uiState.enabled
        )
        appItems(
            apps = uiState.excludedApps,
            focusManager = focusManager,
            onAppClick = onIncludeAppClick,
            onResolveIcon = onResolveIcon,
            enabled = uiState.enabled,
            excluded = true
        )
        spacer(index = 1)
    }
    systemAppsToggle(
        showSystemApps = uiState.showSystemApps,
        onShowSystemAppsClick = onShowSystemAppsClick,
        enabled = uiState.enabled
    )
    headerItem(
        key = SplitTunnelingContentKey.INCLUDED_APPLICATIONS,
        textId = R.string.all_applications,
        enabled = uiState.enabled
    )
    appItems(
        apps = uiState.includedApps,
        focusManager = focusManager,
        onAppClick = onExcludeAppClick,
        onResolveIcon = onResolveIcon,
        enabled = uiState.enabled,
        excluded = false
    )
}

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.appItems(
    apps: List<AppData>,
    focusManager: FocusManager,
    onAppClick: (String) -> Unit,
    onResolveIcon: (String) -> Bitmap?,
    enabled: Boolean,
    excluded: Boolean
) {
    itemsIndexedWithDivider(
        items = apps,
        key = { _, listItem -> listItem.packageName },
        contentType = { _, _ -> ContentType.ITEM }
    ) { index, listItem ->
        SplitTunnelingCell(
            title = listItem.name,
            packageName = listItem.packageName,
            isSelected = excluded,
            enabled = enabled,
            modifier =
                Modifier.animateItemPlacement()
                    .fillMaxWidth()
                    .alpha(
                        if (enabled) {
                            AlphaVisible
                        } else {
                            AlphaDisabled
                        }
                    ),
            onResolveIcon = onResolveIcon
        ) {
            // Move focus down unless the clicked item was the last in this
            // section.
            if (index < apps.size - 1) {
                focusManager.moveFocus(FocusDirection.Down)
            } else {
                focusManager.moveFocus(FocusDirection.Up)
            }

            onAppClick(listItem.packageName)
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.headerItem(key: String, textId: Int, enabled: Boolean) {
    itemWithDivider(key = key, contentType = ContentType.HEADER) {
        HeaderCell(
            modifier =
                Modifier.animateItemPlacement()
                    .alpha(
                        if (enabled) {
                            AlphaVisible
                        } else {
                            AlphaDisabled
                        }
                    ),
            text = stringResource(id = textId),
            background = MaterialTheme.colorScheme.primary
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.systemAppsToggle(
    showSystemApps: Boolean,
    onShowSystemAppsClick: (show: Boolean) -> Unit,
    enabled: Boolean
) {
    itemWithDivider(
        key = SplitTunnelingContentKey.SHOW_SYSTEM_APPLICATIONS,
        contentType = ContentType.OTHER_ITEM
    ) {
        HeaderSwitchComposeCell(
            title = stringResource(id = R.string.show_system_apps),
            isToggled = showSystemApps,
            onCellClicked = { newValue -> onShowSystemAppsClick(newValue) },
            isEnabled = enabled,
            modifier =
                Modifier.animateItemPlacement()
                    .alpha(
                        if (enabled) {
                            AlphaVisible
                        } else {
                            AlphaDisabled
                        }
                    )
        )
    }
}

@OptIn(ExperimentalFoundationApi::class)
private fun LazyListScope.spacer(index: Int) {
    item(key = CommonContentKey.SPACER + index, contentType = ContentType.SPACER) {
        Spacer(modifier = Modifier.animateItemPlacement().height(Dimens.mediumPadding))
    }
}
