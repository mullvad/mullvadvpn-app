package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ContentCopy
import androidx.compose.material.icons.filled.Share
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.preview.ViewLogsUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CopyToClipboardHandle
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.provider.getLogsShareIntent
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview("Content|Loading")
@Composable
private fun PreviewViewLogsScreen(
    @PreviewParameter(ViewLogsUiStatePreviewParameterProvider::class) state: ViewLogsUiState
) {
    AppTheme { ViewLogsScreen(state = state, {}) }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ViewLogs(navigator: DestinationsNavigator) {
    val vm = koinViewModel<ViewLogsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    ViewLogsScreen(state = state, onBackClick = dropUnlessResumed { navigator.navigateUp() })
}

@Composable
fun ViewLogsScreen(state: ViewLogsUiState, onBackClick: () -> Unit) {
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
    state: ViewLogsUiState,
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
                onClick = { clipboardHandle(state.text(), clipboardToastMessage) },
                modifier = Modifier.focusProperties { down = FocusRequester.Cancel },
            ) {
                Icon(
                    imageVector = Icons.Default.ContentCopy,
                    contentDescription = stringResource(id = R.string.copy),
                )
            }
            IconButton(
                onClick = { scope.launch { shareText(context, state.text()) } },
                modifier = Modifier.focusProperties { down = FocusRequester.Cancel },
            ) {
                Icon(
                    imageVector = Icons.Default.Share,
                    contentDescription = stringResource(id = R.string.share),
                )
            }
        },
    )
}

@Composable
private fun Content(state: ViewLogsUiState, paddingValues: PaddingValues) {
    Card(
        modifier =
            Modifier.fillMaxSize()
                .padding(paddingValues)
                .padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin,
                ),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.onPrimary),
    ) {
        if (state.isLoading) {
            MullvadCircularProgressIndicatorMedium(
                modifier =
                    Modifier.padding(Dimens.mediumPadding).align(Alignment.CenterHorizontally),
                color = MaterialTheme.colorScheme.primary,
            )
        } else {
            val listState = rememberLazyListState()
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
                items(state.allLines) { text ->
                    Text(
                        text = text,
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    }
}

private fun shareText(context: Context, logContent: String) {
    val shareIntent = context.getLogsShareIntent(logContent)
    context.startActivity(shareIntent)
}
