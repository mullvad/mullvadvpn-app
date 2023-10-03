package net.mullvad.mullvadvpn.compose.component

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
import androidx.compose.material3.MediumTopAppBar
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import androidx.compose.ui.text.style.TextOverflow
import com.google.accompanist.systemuicontroller.rememberSystemUiController
import net.mullvad.mullvadvpn.lib.theme.AlphaTopBar

@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    statusBarColor: Color,
    navigationBarColor: Color,
    modifier: Modifier = Modifier,
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
    content: @Composable (modifier: Modifier, lazyListState: LazyListState) -> Unit
) {

    val appBarState = rememberTopAppBarState()
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(
            appBarState,
            canScroll = { lazyListState.canScrollBackward || lazyListState.canScrollForward }
        )
    Scaffold(
        modifier = modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MediumTopAppBar(
                title = { Text(appBarTitle, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                navigationIcon = navigationIcon,
                scrollBehavior = scrollBehavior,
                colors =
                    TopAppBarDefaults.mediumTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background
                    ),
                actions = actions
            )
        },
        content = {
            content(Modifier.fillMaxSize().padding(it).drawVerticalScrollbar(lazyListState), lazyListState)
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
    content: @Composable (modifier: Modifier) -> Unit
) {
    val appBarState = rememberTopAppBarState()
    val scrollState = rememberScrollState()
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(
            appBarState,
            canScroll = { scrollState.canScrollBackward || scrollState.canScrollForward }
        )
    Scaffold(
        modifier = modifier.fillMaxSize().nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MediumTopAppBar(
                title = { Text(appBarTitle, maxLines = 1, overflow = TextOverflow.Ellipsis) },
                navigationIcon = navigationIcon,
                scrollBehavior = scrollBehavior,
                colors =
                    TopAppBarDefaults.mediumTopAppBarColors(
                        containerColor = MaterialTheme.colorScheme.background
                    ),
                actions = actions
            )
        },
        content = {
            content(
                Modifier.fillMaxSize().padding(it).drawVerticalScrollbar(scrollState).verticalScroll(scrollState)
            )
        }
    )
}
