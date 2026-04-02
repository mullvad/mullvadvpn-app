package net.mullvad.mullvadvpn.lib.ui.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.grid.LazyGridState
import androidx.compose.foundation.lazy.grid.rememberLazyGridState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FabPosition
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar

@Composable
fun ScaffoldWithTopBar(
    topBarColor: Color,
    modifier: Modifier = Modifier,
    iconTintColor: Color = MaterialTheme.colorScheme.onPrimary,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    isIconAndLogoVisible: Boolean = true,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    enabled: Boolean = true,
    content: @Composable (PaddingValues) -> Unit,
) {
    Scaffold(
        modifier = modifier,
        topBar = {
            MullvadTopBar(
                containerColor = topBarColor,
                iconTintColor = iconTintColor,
                onSettingsClicked = onSettingsClicked,
                onAccountClicked = onAccountClicked,
                isIconAndLogoVisible = isIconAndLogoVisible,
                enabled = enabled,
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        content = content,
    )
}

@Composable
fun ScaffoldWithTopBarAndDeviceName(
    topBarColor: Color,
    modifier: Modifier = Modifier,
    iconTintColor: Color = MaterialTheme.colorScheme.onPrimary,
    onSettingsClicked: (() -> Unit)?,
    onAccountClicked: (() -> Unit)?,
    isIconAndLogoVisible: Boolean = true,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    deviceName: String?,
    timeLeft: Long?,
    content: @Composable (PaddingValues) -> Unit,
) {
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
                    daysLeftUntilExpiry = timeLeft,
                )
            }
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        content = content,
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithSmallTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    bottomBar: @Composable () -> Unit = {},
    floatingActionButton: @Composable () -> Unit = {},
    floatingActionButtonPosition: FabPosition = FabPosition.End,
    content: @Composable (modifier: Modifier) -> Unit,
) {
    Scaffold(
        modifier = modifier.fillMaxSize().imePadding(),
        topBar = {
            MullvadSmallTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions = actions,
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        bottomBar = bottomBar,
        floatingActionButton = floatingActionButton,
        floatingActionButtonPosition = floatingActionButtonPosition,
        content = { content(Modifier.fillMaxSize().padding(it)) },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithSmallTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    lazyListState: LazyListState = rememberLazyListState(),
    scrollbarColor: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    content: @Composable (modifier: Modifier, lazyListState: LazyListState) -> Unit,
) {
    Scaffold(
        modifier = modifier.fillMaxSize().imePadding(),
        topBar = {
            MullvadSmallTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions = actions,
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        content = {
            content(
                Modifier.fillMaxSize()
                    .padding(it)
                    .drawVerticalScrollbar(state = lazyListState, color = scrollbarColor),
                lazyListState,
            )
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithSmallTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    lazyGridState: LazyGridState = rememberLazyGridState(),
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    content: @Composable (modifier: Modifier, lazyGridState: LazyGridState) -> Unit,
) {
    Scaffold(
        modifier = modifier.fillMaxSize().imePadding(),
        topBar = {
            MullvadSmallTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions = actions,
            )
        },
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        content = { content(Modifier.fillMaxSize().padding(it), lazyGridState) },
    )
}
