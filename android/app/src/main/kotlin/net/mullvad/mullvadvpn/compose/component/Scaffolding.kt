package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.OpenInNew
import androidx.compose.material.icons.filled.OpenInNew
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.material3.rememberTopAppBarState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.nestedscroll.nestedScroll
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar

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

@Composable
fun MullvadSnackbar(modifier: Modifier = Modifier, snackbarData: SnackbarData) {
    Snackbar(
        modifier = modifier,
        snackbarData = snackbarData,
        containerColor = MaterialTheme.colorScheme.surfaceContainer,
        contentColor = MaterialTheme.colorScheme.onSurface,
        actionColor = MaterialTheme.colorScheme.onSurface,
    )
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
fun ScaffoldWithMediumTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    lazyListState: LazyListState = rememberLazyListState(),
    scrollbarColor: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    content: @Composable (modifier: Modifier, lazyListState: LazyListState) -> Unit,
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
                scrollBehavior = scrollBehavior,
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
fun ScaffoldWithMediumTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    scrollbarColor: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    content: @Composable (modifier: Modifier) -> Unit,
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
                scrollBehavior = scrollBehavior,
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
                    .drawVerticalScrollbar(state = scrollState, color = scrollbarColor)
                    .verticalScroll(scrollState)
            )
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ScaffoldWithLargeTopBarAndButton(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    onButtonClick: () -> Unit = {}, // Add button
    buttonTitle: String,
    scrollbarColor: Color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
    content: @Composable (modifier: Modifier) -> Unit,
) {
    val appBarState = rememberTopAppBarState()
    val scrollState = rememberScrollState()
    val canScroll = scrollState.canScrollForward || scrollState.canScrollBackward
    val scrollBehavior =
        TopAppBarDefaults.exitUntilCollapsedScrollBehavior(appBarState, canScroll = { canScroll })
    Scaffold(
        modifier =
            modifier
                .fillMaxSize()
                .systemBarsPadding()
                .nestedScroll(scrollBehavior.nestedScrollConnection),
        topBar = {
            MullvadLargeTopBar(
                title = appBarTitle,
                navigationIcon = navigationIcon,
                actions,
                scrollBehavior = scrollBehavior,
            )
        },
        bottomBar = {
            PrimaryButton(
                text = buttonTitle,
                onClick = onButtonClick,
                modifier =
                    Modifier.padding(
                        horizontal = Dimens.sideMargin,
                        vertical = Dimens.screenVerticalMargin,
                    ),
                trailingIcon = {
                    Icon(
                        imageVector = Icons.AutoMirrored.Filled.OpenInNew,
                        tint = MaterialTheme.colorScheme.onPrimary,
                        contentDescription = null,
                    )
                },
            )
        },
        content = {
            content(
                Modifier.fillMaxSize()
                    .padding(it)
                    .drawVerticalScrollbar(state = scrollState, color = scrollbarColor)
                    .verticalScroll(scrollState)
            )
        },
    )
}

@Composable
fun ScaffoldWithSmallTopBar(
    appBarTitle: String,
    modifier: Modifier = Modifier,
    navigationIcon: @Composable () -> Unit = {},
    actions: @Composable RowScope.() -> Unit = {},
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
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
        content = { content(Modifier.fillMaxSize().padding(it)) },
    )
}
