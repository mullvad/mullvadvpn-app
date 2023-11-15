package net.mullvad.mullvadvpn.compose.screen

import android.content.Intent
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
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorMedium
import net.mullvad.mullvadvpn.compose.component.MullvadMediumTopBar
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState

@Preview
@Composable
private fun PreviewViewLogsScreen() {
    AppTheme { ViewLogsScreen(uiState = ViewLogsUiState(listOf("Lorem ipsum"))) }
}

@Preview
@Composable
private fun PreviewViewLogsLoadingScreen() {
    AppTheme { ViewLogsScreen(uiState = ViewLogsUiState()) }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ViewLogsScreen(
    uiState: ViewLogsUiState,
    onBackClick: () -> Unit = {},
) {
    val context = LocalContext.current
    val shareLogs: (logs: String) -> Unit = {
        val sendIntent: Intent =
            Intent().apply {
                action = Intent.ACTION_SEND
                putExtra(Intent.EXTRA_TEXT, uiState.allLines.joinToString("\n"))
                type = "text/plain"
            }
        val shareIntent = Intent.createChooser(sendIntent, null)

        context.startActivity(shareIntent)
    }

    Scaffold(
        topBar = {
            MullvadMediumTopBar(
                title = stringResource(id = R.string.view_logs),
                navigationIcon = { NavigateBackIconButton(onBackClick) },
                actions = {
                    IconButton(onClick = { shareLogs(uiState.allLines.joinToString("\n")) }) {
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
            if (uiState.isLoading) {
                MullvadCircularProgressIndicatorMedium(
                    modifier =
                        Modifier.padding(Dimens.mediumPadding).align(Alignment.CenterHorizontally),
                    color = MaterialTheme.colorScheme.primary
                )
            } else {
                val state = rememberLazyListState()
                LazyColumn(
                    state = state,
                    modifier =
                        Modifier.drawVerticalScrollbar(
                                state,
                                MaterialTheme.colorScheme.primary.copy(alpha = AlphaScrollbar)
                            )
                            .padding(horizontal = Dimens.smallPadding)
                ) {
                    items(uiState.allLines) {
                        Text(text = it, style = MaterialTheme.typography.bodySmall)
                    }
                }
            }
        }
    }
}
