package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import me.onebone.toolbar.CollapsingToolbarScaffold
import me.onebone.toolbar.CollapsingToolbarScaffoldScope
import me.onebone.toolbar.CollapsingToolbarScaffoldState
import me.onebone.toolbar.CollapsingToolbarScope
import me.onebone.toolbar.ExperimentalToolbarApi
import me.onebone.toolbar.ScrollStrategy
import net.mullvad.mullvadvpn.lib.theme.AlphaTopBar

@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color,
    iconTintColor: Color = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaTopBar),
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    isIconAndLogoVisible: Boolean = true,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    content: @Composable (PaddingValues) -> Unit,
) {
    val systemUiController = rememberSystemUiController()
    LaunchedEffect(key1 = statusBarColor, key2 = navigationBarColor) {
        systemUiController.setStatusBarColor(statusBarColor)
        systemUiController.setNavigationBarColor(navigationBarColor)
    }

    Scaffold(
        topBar = {
            TopBar(
                backgroundColor = topBarColor,
                iconTintColor = iconTintColor,
                onSettingsClicked = onSettingsClicked,
                onAccountClicked = onAccountClicked,
                isIconAndLogoVisible = isIconAndLogoVisible
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) }
            )
        },
        content = content
    )
}

@Composable
fun MullvadSnackbar(snackbarData: SnackbarData) {
    Snackbar(snackbarData = snackbarData, contentColor = MaterialTheme.colorScheme.secondary)
}

@Composable
@OptIn(ExperimentalToolbarApi::class)
fun CollapsableAwareToolbarScaffold(
    backgroundColor: Color,
    modifier: Modifier = Modifier,
    state: CollapsingToolbarScaffoldState,
    scrollStrategy: ScrollStrategy,
    isEnabledWhenCollapsable: Boolean = true,
    toolbarModifier: Modifier = Modifier,
    toolbar: @Composable CollapsingToolbarScope.() -> Unit,
    body: @Composable CollapsingToolbarScaffoldScope.() -> Unit
) {
    val systemUiController = rememberSystemUiController()
    systemUiController.setStatusBarColor(backgroundColor)
    systemUiController.setNavigationBarColor(backgroundColor)

    var isCollapsable by remember { mutableStateOf(false) }

    LaunchedEffect(isCollapsable) {
        if (!isCollapsable) {
            state.toolbarState.expand()
        }
    }

    CollapsingToolbarScaffold(
        modifier = modifier.background(backgroundColor),
        state = state,
        scrollStrategy = scrollStrategy,
        enabled = isEnabledWhenCollapsable && isCollapsable,
        toolbarModifier = toolbarModifier,
        toolbar = toolbar,
        body = {
            var bodyHeight by remember { mutableIntStateOf(0) }

            BoxWithConstraints(
                modifier = Modifier.onGloballyPositioned { bodyHeight = it.size.height }
            ) {
                val minMaxToolbarHeightDiff =
                    with(state) { toolbarState.maxHeight - toolbarState.minHeight }
                val isContentHigherThanCollapseThreshold =
                    with(LocalDensity.current) {
                        bodyHeight > maxHeight.toPx() - minMaxToolbarHeightDiff
                    }
                isCollapsable = isContentHigherThanCollapseThreshold
                body()
            }
        }
    )
}
