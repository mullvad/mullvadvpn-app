package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ContentCopy
import androidx.compose.material.icons.rounded.Share
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.preview.ViewLogsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.util.CopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.core.animation.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.ui.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadSnackbar
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.provider.getLogsShareIntent
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Loading")
@Composable
private fun PreviewViewLogsScreen(
    @PreviewParameter(ViewLogsUiStatePreviewParameterProvider::class)
    state: Lc<Unit, ViewLogsUiState>
) {
    AppTheme { ViewLogsScreen(state = state, onBackClick = {}) }
}

@Destination<MainGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ViewLogs(navigator: DestinationsNavigator) {
    val vm = koinViewModel<ViewLogsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    ViewLogsScreen(state = state, onBackClick = dropUnlessResumed { navigator.navigateUp() })
}

@Composable
fun ViewLogsScreen(state: Lc<Unit, ViewLogsUiState>, onBackClick: () -> Unit) {
    val snackbarHostState = remember { SnackbarHostState() }
    val clipboardHandle =
        createCopyToClipboardHandle(snackbarHostState = snackbarHostState, isSensitive = false)
    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) },
            )
        },
        topBar = { TopBar(state, clipboardHandle, onBackClick) },
    ) {
        Content(state, it)
    }
}

@OptIn(ExperimentalComposeUiApi::class, ExperimentalMaterial3Api::class)
@Composable
private fun TopBar(
    state: Lc<Unit, ViewLogsUiState>,
    clipboardHandle: CopyToClipboardHandle,
    onBackClick: () -> Unit,
) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    MullvadMediumTopBar(
        title = stringResource(id = R.string.view_logs),
        navigationIcon = {
            NavigateBackIconButton(
                onNavigateBack = onBackClick,
                modifier = Modifier.focusProperties { down = FocusRequester.Cancel },
            )
        },
        actions = {
            val clipboardToastMessage = stringResource(R.string.copied_logs_to_clipboard)
            IconButton(
                onClick = {
                    clipboardHandle(state.contentOrNull()?.text() ?: "", clipboardToastMessage)
                },
                modifier = Modifier.focusProperties { down = FocusRequester.Cancel },
                enabled = state is Lc.Content,
            ) {
                Icon(
                    imageVector = Icons.Rounded.ContentCopy,
                    contentDescription = stringResource(id = R.string.copy),
                )
            }
            IconButton(
                onClick = {
                    scope.launch { shareText(context, state.contentOrNull()?.text() ?: "") }
                },
                modifier = Modifier.focusProperties { down = FocusRequester.Cancel },
                enabled = state is Lc.Content,
            ) {
                Icon(
                    imageVector = Icons.Rounded.Share,
                    contentDescription = stringResource(id = R.string.share),
                )
            }
        },
    )
}

@Composable
private fun Content(state: Lc<Unit, ViewLogsUiState>, paddingValues: PaddingValues) {
    Card(
        modifier =
            Modifier.fillMaxSize()
                .padding(paddingValues)
                .padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenBottomMargin,
                ),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.onPrimary),
    ) {
        when (state) {
            is Lc.Loading -> Loading()
            is Lc.Content -> Content(state.value.allLines)
        }
    }
}

@Composable
private fun ColumnScope.Loading() {
    MullvadCircularProgressIndicatorMedium(
        modifier = Modifier.padding(Dimens.mediumPadding).align(Alignment.CenterHorizontally),
        color = MaterialTheme.colorScheme.primary,
    )
}

@Composable
private fun Content(allLines: List<String>) {
    val listState = rememberLazyListState()
    // Logs are always in English and should be Ltr
    CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
        LazyColumn(
            state = listState,
            modifier =
                Modifier.fillMaxWidth()
                    .drawVerticalScrollbar(
                        listState,
                        MaterialTheme.colorScheme.primary.copy(alpha = AlphaScrollbar),
                    )
                    .padding(horizontal = Dimens.smallPadding),
        ) {
            items(allLines) { text ->
                Text(
                    text = text,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        }
    }
}

private fun shareText(context: Context, logContent: String) {
    val shareIntent = context.getLogsShareIntent(logContent)
    context.startActivity(shareIntent)
}
