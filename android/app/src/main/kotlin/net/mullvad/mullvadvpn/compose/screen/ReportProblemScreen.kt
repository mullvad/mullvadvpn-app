package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.CollapsingToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.dialog.ReportProblemNoEmailDialog
import net.mullvad.mullvadvpn.compose.dialog.ShowReportProblemStateDialog
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ReportProblemUiState
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState

@Preview
@Composable
private fun ReportProblemScreenPreview() {
    AppTheme { ReportProblemScreen(uiState = ReportProblemUiState()) }
}

@Preview
@Composable
private fun ReportProblemSendingScreenPreview() {
    AppTheme {
        ReportProblemScreen(uiState = ReportProblemUiState(false, SendingReportUiState.Sending))
    }
}

@Preview
@Composable
private fun ReportProblemConfirmNoEmailScreenPreview() {
    AppTheme { ReportProblemScreen(uiState = ReportProblemUiState(true)) }
}

@Preview
@Composable
private fun ReportProblemSuccessScreenPreview() {
    AppTheme {
        ReportProblemScreen(
            uiState = ReportProblemUiState(false, SendingReportUiState.Success(null))
        )
    }
}

@Preview
@Composable
private fun ReportProblemErrorScreenPreview() {
    AppTheme {
        ReportProblemScreen(
            uiState =
                ReportProblemUiState(
                    false,
                    SendingReportUiState.Error(SendProblemReportResult.Error.CollectLog)
                )
        )
    }
}

@Composable
fun ReportProblemScreen(
    uiState: ReportProblemUiState,
    onSendReport: (String, String) -> Unit = { _, _ -> },
    onDismissNoEmailDialog: () -> Unit = {},
    onClearSendResult: () -> Unit = {},
    onNavigateToViewLogs: () -> Unit = {},
    onBackClick: () -> Unit = {}
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
                backgroundColor = MaterialTheme.colorScheme.background,
                onBackClicked = onBackClick,
                title = stringResource(id = R.string.report_a_problem),
                progress = progress,
                modifier = scaffoldModifier,
            )
        },
    ) {
        var email by rememberSaveable { mutableStateOf("") }
        var description by rememberSaveable { mutableStateOf("") }

        // Dialog to show sending states
        if (uiState.sendingState != null) {
            ShowReportProblemStateDialog(
                uiState.sendingState,
                onDismiss = onClearSendResult,
                onClearForm = {
                    email = ""
                    description = ""
                },
                retry = { onSendReport(email, description) }
            )
        }

        // Dialog to show confirm if no email was added
        if (uiState.showConfirmNoEmail) {
            ReportProblemNoEmailDialog(
                onDismiss = onDismissNoEmailDialog,
                onConfirm = { onSendReport(email, description) }
            )
        }

        Surface(color = MaterialTheme.colorScheme.background) {
            Column(
                modifier =
                    Modifier.padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.verticalSpace,
                    ),
                verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding)
            ) {
                Text(text = stringResource(id = R.string.problem_report_description))

                TextField(
                    modifier = Modifier.fillMaxWidth(),
                    value = email,
                    onValueChange = { email = it },
                    maxLines = 1,
                    singleLine = true,
                    placeholder = { Text(text = stringResource(id = R.string.user_email_hint)) },
                    colors = mullvadWhiteTextFieldColors()
                )

                TextField(
                    modifier = Modifier.fillMaxWidth().weight(1f),
                    value = description,
                    onValueChange = { description = it },
                    placeholder = { Text(stringResource(R.string.user_message_hint)) },
                    colors = mullvadWhiteTextFieldColors()
                )

                ActionButton(
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.primary,
                            contentColor = MaterialTheme.colorScheme.onPrimary
                        ),
                    onClick = onNavigateToViewLogs,
                    text = stringResource(id = R.string.view_logs)
                )

                ActionButton(
                    colors =
                        ButtonDefaults.buttonColors(
                            containerColor = MaterialTheme.colorScheme.surface,
                            contentColor = MaterialTheme.colorScheme.onSurface
                        ),
                    onClick = { onSendReport(email, description) },
                    isEnabled = description.isNotEmpty(),
                    text = stringResource(id = R.string.send)
                )
            }
        }
    }
}
