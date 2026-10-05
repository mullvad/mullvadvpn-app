package net.mullvad.mullvadvpn.feature.appearance.impl

import android.app.StatusBarManager
import android.content.ComponentName
import android.graphics.drawable.Icon
import android.os.Build
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.content.getSystemService
import androidx.lifecycle.compose.dropUnlessResumed
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.appicon.api.AppIconNavKey
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.lib.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAppearanceScreen() {
    AppTheme { AppearanceScreen(onAppIconClick = {}, onLanguageClick = {}, onBackClick = {}) }
}

@Composable
fun Appearance(navigator: Navigator) {
    AppearanceScreen(
        onAppIconClick = dropUnlessResumed { navigator.navigate(AppIconNavKey) },
        onLanguageClick =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                dropUnlessResumed { navigator.navigate(LanguageNavKey) }
            } else {
                null
            },
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AppearanceScreen(
    onAppIconClick: () -> Unit,
    onLanguageClick: (() -> Unit)?,
    onBackClick: () -> Unit,
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.appearance),
        navigationIcon = {
            unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) }
        },
    ) { modifier ->
        val lazyListState: LazyListState = rememberLazyListState()

        val context = LocalContext.current
        val statusBarManager = context.getSystemService<StatusBarManager>()

        LazyColumn(
            modifier = modifier.padding(horizontal = Dimens.sideMarginNew),
            state = lazyListState,
        ) {
            itemWithDivider {
                NavigationListItem(
                    title = stringResource(id = R.string.app_icon),
                    onClick = onAppIconClick,
                    position = if (onLanguageClick != null) Position.Top else Position.Single,
                )
            }
            if (onLanguageClick != null) {
                item {
                    NavigationListItem(
                        title = stringResource(id = R.string.language),
                        onClick = onLanguageClick,
                        position = Position.Bottom,
                    )
                }
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    item {
                        NavigationListItem(
                            title = "Add tile",
                            onClick = {
                                statusBarManager!!.requestAddTileService(
                                    ComponentName(context, "net.mullvad.mullvadvpn.app.tile.MullvadTileService"),
                                    "sequence",
                                    Icon.createWithResource(context, R.drawable.small_logo_white),
                                    context.mainExecutor,
                                ) {
                                    Logger.d { "Add tile result: $it" }
                                    when (it) {
                                        StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ALREADY_ADDED,
                                        StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ADDED -> {}
                                        StatusBarManager.TILE_ADD_REQUEST_ERROR_MISMATCHED_PACKAGE,
                                        StatusBarManager.TILE_ADD_REQUEST_ERROR_REQUEST_IN_PROGRESS,
                                        StatusBarManager.TILE_ADD_REQUEST_ERROR_BAD_COMPONENT,
                                        StatusBarManager.TILE_ADD_REQUEST_ERROR_NOT_CURRENT_USER,
                                        StatusBarManager
                                            .TILE_ADD_REQUEST_ERROR_APP_NOT_IN_FOREGROUND,
                                        StatusBarManager
                                            .TILE_ADD_REQUEST_ERROR_NO_STATUS_BAR_SERVICE -> {}
                                    }
                                }
                            },
                            position = Position.Bottom,
                        )
                    }
                }
            }
        }
    }
}
