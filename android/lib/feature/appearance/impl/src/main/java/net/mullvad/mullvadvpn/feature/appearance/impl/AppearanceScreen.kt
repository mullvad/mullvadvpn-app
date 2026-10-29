package net.mullvad.mullvadvpn.feature.appearance.impl

import android.app.StatusBarManager
import android.content.ComponentName
import android.content.Context
import android.graphics.drawable.Icon as AndroidIcon
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.rounded.OpenInNew
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.content.ContextCompat
import androidx.core.content.getSystemService
import androidx.lifecycle.compose.dropUnlessResumed
import co.touchlab.kermit.Logger
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.appearance.impl.AppearanceSideEffect.RequestAddTileResult
import net.mullvad.mullvadvpn.feature.appicon.api.AppIconNavKey
import net.mullvad.mullvadvpn.feature.language.api.LanguageNavKey
import net.mullvad.mullvadvpn.lib.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.common.compose.itemWithDivider
import net.mullvad.mullvadvpn.lib.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.lib.common.constant.TILE_SERVICE_CLASS
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.listitem.NavigationListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ListItemInfo
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewAppearanceScreen() {
    AppTheme {
        AppearanceScreen(
            onAppIconClick = {},
            onLanguageClick = {},
            onBackClick = {},
            onRequestAddTileResult = {},
        )
    }
}

@Composable
fun Appearance(navigator: Navigator) {
    val vm = koinViewModel<AppearanceViewModel>()
    val snackbarHostState = remember { SnackbarHostState() }
    val resources = LocalResources.current

    CollectSideEffectWithLifecycle(vm.uiSideEffect) { sideEffect ->
        val message =
            when (sideEffect) {
                RequestAddTileResult.TileAlreadyAdded ->
                    resources.getString(R.string.add_tile_already_added)
                RequestAddTileResult.TileAdded -> resources.getString(R.string.add_tile_added)
                RequestAddTileResult.NoStatusBarService ->
                    resources.getString(R.string.add_tile_no_status_bar)
                RequestAddTileResult.NotCurrentUser -> resources.getString(R.string.add_tile_error)
                is RequestAddTileResult.Unknown ->
                    resources.getString(
                        R.string.add_tile_generic_error,
                        sideEffect.result,
                    )
            }
        launch { snackbarHostState.showSnackbarImmediately(message = message) }
    }

    AppearanceScreen(
        onAppIconClick = dropUnlessResumed { navigator.navigate(AppIconNavKey) },
        onLanguageClick =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                dropUnlessResumed { navigator.navigate(LanguageNavKey) }
            } else {
                null
            },
        onBackClick = dropUnlessResumed { navigator.goBack() },
        onRequestAddTileResult = vm::onRequestAddTileResult,
        snackbarHostState = snackbarHostState,
    )
}

@RequiresApi(Build.VERSION_CODES.TIRAMISU)
private fun Context.requestAddTile(resultCallback: (Int) -> Unit) {
    val statusBarManager = getSystemService<StatusBarManager>()
    statusBarManager?.requestAddTileService(
        ComponentName(packageName, TILE_SERVICE_CLASS),
        getString(R.string.toggle_vpn),
        AndroidIcon.createWithResource(this, R.drawable.small_logo_black),
        ContextCompat.getMainExecutor(this),
        resultCallback,
    ) ?: Logger.w("Failed to get StatusBarManager system service")
}

@Composable
fun AppearanceScreen(
    onAppIconClick: () -> Unit,
    onLanguageClick: (() -> Unit)?,
    onBackClick: () -> Unit,
    onRequestAddTileResult: (Int) -> Unit,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.appearance),
        snackbarHostState = snackbarHostState,
        navigationIcon = {
            unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) }
        },
    ) { modifier ->
        val lazyListState: LazyListState = rememberLazyListState()

        val context = LocalContext.current
        LazyColumn(
            modifier = modifier.fillMaxSize().padding(horizontal = Dimens.sideMarginNew),
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
            }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                item { Spacer(Modifier.height(Dimens.cellVerticalSpacing)) }

                item {
                    MullvadListItem(
                        onClick = { context.requestAddTile(onRequestAddTileResult) },
                        trailingContent = {
                            Icon(
                                imageVector = Icons.AutoMirrored.Rounded.OpenInNew,
                                contentDescription = null,
                            )
                        },
                    ) {
                        Text(text = stringResource(id = R.string.add_tile_title))
                    }

                    ListItemInfo(text = stringResource(id = R.string.add_tile_info))
                }
            }
        }
    }
}
