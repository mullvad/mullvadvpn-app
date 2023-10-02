package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.selection.SelectionContainer
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.CollapsingToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ViewLogsUiState

@Preview
@Composable
private fun PreviewViewLogsScreen() {
    AppTheme { ViewLogsScreen(uiState = ViewLogsUiState("Lorem ipsum")) }
}

@Preview
@Composable
private fun PreviewViewLogsLoadingScreen() {
    AppTheme { ViewLogsScreen(uiState = ViewLogsUiState()) }
}

@Composable
fun ViewLogsScreen(
    uiState: ViewLogsUiState,
    onBackClick: () -> Unit = {},
) {

    val scaffoldState = rememberCollapsingToolbarScaffoldState()
    val progress = scaffoldState.toolbarState.progress
    CollapsingToolbarScaffold(
        backgroundColor = MaterialTheme.colorScheme.background,
        modifier = Modifier.fillMaxSize(),
        state = scaffoldState,
        scrollStrategy = ScrollStrategy.ExitUntilCollapsed,
        isEnabledWhenCollapsable = false,
        toolbar = {
            val scaffoldModifier =
                Modifier.road(
                    whenCollapsed = Alignment.TopCenter,
                    whenExpanded = Alignment.BottomStart
                )
            CollapsingTopBar(
                backgroundColor = MaterialTheme.colorScheme.secondary,
                onBackClicked = onBackClick,
                title = stringResource(id = R.string.view_logs),
                progress = progress,
                modifier = scaffoldModifier,
            )
        },
    ) {
        Card(
            modifier =
                Modifier.fillMaxSize()
                    .padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.screenVerticalMargin
                    ),
        ) {
            if (uiState.isLoading) {
                CircularProgressIndicator(
                    modifier =
                        Modifier.padding(Dimens.mediumPadding).align(Alignment.CenterHorizontally)
                )
            } else {
                SelectionContainer {
                    val scrollState = rememberScrollState()
                    Column(
                        modifier =
                            Modifier.drawVerticalScrollbar(
                                scrollState,
                                color = MaterialTheme.colorScheme.primary
                            )
                    ) {
                        TextField(
                            modifier =
                                Modifier.verticalScroll(scrollState)
                                    .padding(horizontal = Dimens.smallPadding),
                            value = uiState.allLines,
                            textStyle = MaterialTheme.typography.bodySmall,
                            onValueChange = {},
                            readOnly = true,
                            colors =
                                TextFieldDefaults.colors(
                                    focusedTextColor = Color.Black,
                                    unfocusedTextColor = Color.Black,
                                    disabledTextColor = Color.Black,
                                    cursorColor = MaterialTheme.colorScheme.background,
                                )
                        )
                    }
                }
            }
        }
    }
}
