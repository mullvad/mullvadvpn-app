package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar

@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color,
    modifier: Modifier = Modifier,
    iconTintColor: Color = MaterialTheme.colorScheme.onPrimary,
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
        modifier = modifier,
        topBar = {
            MullvadTopBar(
                containerColor = topBarColor,
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
fun ScaffoldWithTopBarAndDeviceName(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color?,
    modifier: Modifier = Modifier,
    iconTintColor: Color = MaterialTheme.colorScheme.onPrimary,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    isIconAndLogoVisible: Boolean = true,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    deviceName: String?,
    timeLeft: Int?,
    content: @Composable (PaddingValues) -> Unit,
) {
    val systemUiController = rememberSystemUiController()
    LaunchedEffect(key1 = statusBarColor, key2 = navigationBarColor) {
        systemUiController.setStatusBarColor(statusBarColor)
        if (navigationBarColor != null) {
            systemUiController.setNavigationBarColor(navigationBarColor)
        }
    }

    Scaffold(
        modifier = modifier,
        topBar = {
            Column {
                MullvadTopBarWithDeviceName(
                    containerColor = topBarColor,
                    iconTintColor = iconTintColor,
                    onSettingsClicked = onSettingsClicked,
                    onAccountClicked = onAccountClicked,
                    isIconAndLogoVisible = isIconAndLogoVisible,
                    deviceName = deviceName,
                    daysLeftUntilExpiry = timeLeft
                )
            }
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
@OptIn(ExperimentalMaterial3Api::class)
fun ScaffoldWithMediumTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    lazyListState: LazyListState = rememberLazyListState(),
    scrollbarColor: Color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar),
    content: @Composable (modifier: Modifier, lazyListState: LazyListState) -> Unit
) {

    val appBarState = rememberTopAppBarState()
    val canScroll = lazyListState.canScrollForward || lazyListState.canScrollBackward
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(appBarState, canScroll = { canScroll })
    Scaffold(
        modifier = modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MullvadMediumTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions,
                scrollBehavior = if (canScroll) scrollBehavior else null
            )
        },
        content = {
            content(
                Modifier.fillMaxSize()
                    .padding(it)
                    .drawVerticalScrollbar(state = lazyListState, color = scrollbarColor),
                lazyListState
            )
        }
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithMediumTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    scrollbarColor: Color = MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar),
    content: @Composable (modifier: Modifier) -> Unit
) {
    val appBarState = rememberTopAppBarState()
    val scrollState = rememberScrollState()
    val canScroll = scrollState.canScrollForward || scrollState.canScrollBackward
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(appBarState, canScroll = { canScroll })
    Scaffold(
        modifier = modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MullvadMediumTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions,
                scrollBehavior =
                    if (canScroll) {
                        scrollBehavior
                    } else {
                        null
                    }
            )
        },
        content = {
            content(
                Modifier.fillMaxSize()
                    .padding(it)
                    .drawVerticalScrollbar(state = scrollState, color = scrollbarColor)
                    .verticalScroll(scrollState)
            )
        }
    )
}
