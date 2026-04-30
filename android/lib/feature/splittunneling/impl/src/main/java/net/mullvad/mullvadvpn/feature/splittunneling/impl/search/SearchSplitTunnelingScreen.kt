package net.mullvad.mullvadvpn.feature.splittunneling.impl.search

import android.graphics.drawable.Drawable
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusManager
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalFocusManager
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.splittunneling.impl.CommonContentKey
import net.mullvad.mullvadvpn.feature.splittunneling.impl.ContentType
import net.mullvad.mullvadvpn.feature.splittunneling.impl.SplitTunnelingContentKey
import net.mullvad.mullvadvpn.feature.splittunneling.impl.appItems
import net.mullvad.mullvadvpn.feature.splittunneling.impl.excludedAppsHeaderItem
import net.mullvad.mullvadvpn.feature.splittunneling.impl.getApplicationIconOrNull
import net.mullvad.mullvadvpn.feature.splittunneling.impl.headerItem
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.PackageName
import net.mullvad.mullvadvpn.lib.ui.component.MullvadSearchBar
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel

@Composable
fun SearchSplitTunnelingScreen(navigator: Navigator) {
    val viewModel = koinViewModel<SearchSplitTunnelingViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    SearchSplitTunnelingScreen(
        state = state,
        onSearchInputChanged = viewModel::onSearchInputChanged,
        onExcludeAppClick = viewModel::onExcludeAppClick,
        onIncludeAppClick = viewModel::onIncludeAppClick,
        onGoBack = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun SearchSplitTunnelingScreen(
    state: Lc<Unit, SearchSplitTunnelingUiState>,
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSearchInputChanged: (String) -> Unit,
    onExcludeAppClick: (packageName: PackageName) -> Unit,
    onIncludeAppClick: (packageName: PackageName) -> Unit,
    onGoBack: () -> Unit,
) {
    val keyboardController = LocalSoftwareKeyboardController.current
    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        }
    ) {
        val focusManager = LocalFocusManager.current
        Column(modifier = Modifier.fillMaxSize().padding(it).consumeWindowInsets(it).imePadding()) {
            val focusRequester = remember { FocusRequester() }
            LaunchedEffect(state is Lc.Content) { focusRequester.requestFocus() }
            MullvadSearchBar(
                modifier = Modifier.focusRequester(focusRequester),
                searchTerm = state.contentOrNull()?.searchTerm ?: "",
                enabled = state is Lc.Content,
                onSearchInputChanged = onSearchInputChanged,
                hideKeyboard = { keyboardController?.hide() },
                onGoBack = onGoBack,
            )
            HorizontalDivider(color = MaterialTheme.colorScheme.onSurface)
            val lazyListState = rememberLazyListState()
            val context = LocalContext.current
            val packageManager = remember(context) { context.packageManager }
            LazyColumn(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier =
                    Modifier.fillMaxSize()
                        .padding(horizontal = Dimens.mediumPadding)
                        .background(color = MaterialTheme.colorScheme.surface)
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
            ) {
                when (state) {
                    is Lc.Loading -> {
                        spacer()
                        loading()
                    }
                    is Lc.Content -> {
                        appList(
                            state = state.value,
                            focusManager = focusManager,
                            onExcludeAppClick = onExcludeAppClick,
                            onIncludeAppClick = onIncludeAppClick,
                            onResolveIcon = { packageName ->
                                packageManager.getApplicationIconOrNull(packageName)
                            },
                        )
                    }
                }
            }
        }
    }
}

private fun LazyListScope.loading() {
    item(key = CommonContentKey.PROGRESS, contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge()
    }
}

private fun LazyListScope.spacer() {
    item(contentType = ContentType.SPACER) {
        Spacer(modifier = Modifier.animateItem().height(Dimens.cellVerticalSpacing))
    }
}

private fun LazyListScope.appList(
    state: SearchSplitTunnelingUiState,
    focusManager: FocusManager,
    onExcludeAppClick: (packageName: PackageName) -> Unit,
    onIncludeAppClick: (packageName: PackageName) -> Unit,
    onResolveIcon: (PackageName) -> Drawable?,
) {
    if (state.includedApps.isEmpty() && state.excludedApps.isEmpty()) {
        item { NoAppsMatchingSearch(state.searchTerm) }
    }
    if (state.excludedApps.isNotEmpty()) {
        excludedAppsHeaderItem(
            key = SplitTunnelingContentKey.EXCLUDED_APPLICATIONS,
            textId = R.string.exclude_applications,
            enabled = true,
            exludedAppsCount = state.excludedApps.size,
            includedAppsCount = state.includedApps.size,
        )
        appItems(
            apps = state.excludedApps,
            focusManager = focusManager,
            onAppClick = onIncludeAppClick,
            onResolveIcon = onResolveIcon,
            enabled = true,
            excluded = true,
        )
        spacer()
    }
    if (state.includedApps.isNotEmpty()) {
        headerItem(
            key = SplitTunnelingContentKey.INCLUDED_APPLICATIONS,
            textId = R.string.all_applications,
            enabled = true,
        )
        appItems(
            apps = state.includedApps,
            focusManager = focusManager,
            onAppClick = onExcludeAppClick,
            onResolveIcon = onResolveIcon,
            enabled = true,
            excluded = false,
        )
        spacer()
    }
}

@Composable
private fun NoAppsMatchingSearch(searchTerm: String) {
    Text(
        text = stringResource(R.string.search_no_matches_for_text, searchTerm),
        style = MaterialTheme.typography.bodyMedium,
        textAlign = TextAlign.Center,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        maxLines = 2,
        overflow = TextOverflow.Ellipsis,
        modifier = Modifier.padding(Dimens.cellVerticalSpacing),
    )
}
