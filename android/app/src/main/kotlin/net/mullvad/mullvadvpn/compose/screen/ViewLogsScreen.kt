package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Card
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.createCopyToClipboardHandle
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.provider.getLogsShareIntent
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState
import net.mullvad.mullvadvpn.viewmodel.ViewLogsViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewViewLogsScreen() {
    AppTheme { ViewLogsScreen(state = ViewLogsUiState(listOf("Lorem ipsum"))) }
}

@Preview
@Composable
private fun PreviewViewLogsLoadingScreen() {
    AppTheme { ViewLogsScreen(state = ViewLogsUiState()) }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun ViewLogs(navigator: DestinationsNavigator) {
    val vm = koinViewModel<ViewLogsViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()
    ViewLogsScreen(state = state, onBackClick = navigator::navigateUp)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ViewLogsScreen(
    state: ViewLogsUiState,
    onBackClick: () -> Unit = {},
) {
    val context = LocalContext.current

    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()
    val clipboardHandle = createCopyToClipboardHandle(snackbarHostState = snackbarHostState)
    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) }
            )
        },
        topBar = {
            MullvadMediumTopBar(
                title = stringResource(id = R.string.view_logs),
                navigationIcon = { NavigateBackIconButton(onBackClick) },
                actions = {
                    val clipboardToastMessage = stringResource(R.string.copied_logs_to_clipboard)
                    IconButton(onClick = { clipboardHandle(state.text(), clipboardToastMessage) }) {
                        Icon(
                            painter = painterResource(id = R.drawable.icon_copy),
                            contentDescription = null
                        )
                    }
                    IconButton(onClick = { scope.launch { shareText(context, state.text()) } }) {
                        Icon(imageVector = Icons.Default.Share, contentDescription = null)
                    }
                }
            )
        }
    ) {
        Card(
            modifier =
                Modifier.fillMaxSize()
                    .padding(it)
                    .padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
        ) {
            if (state.isLoading) {
                MullvadCircularProgressIndicatorMedium(
                    modifier =
                        Modifier.padding(Dimens.mediumPadding).align(Alignment.CenterHorizontally),
                    color = MaterialTheme.colorScheme.primary
                )
            } else {
                val listState = rememberLazyListState()
                LazyColumn(
                    state = listState,
                    modifier =
                        Modifier.drawVerticalScrollbar(
                                listState,
                                MaterialTheme.colorScheme.primary.copy(alpha = AlphaScrollbar)
                            )
                            .padding(horizontal = Dimens.smallPadding)
                ) {
                    items(state.allLines) {
                        Text(text = it, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}

private fun shareText(context: Context, logContent: String) {
    val shareIntent = context.getLogsShareIntent("Share logs", logContent)
    context.startActivity(shareIntent)
}
