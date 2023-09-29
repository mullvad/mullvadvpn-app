package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import me.onebone.toolbar.ScrollStrategy
import me.onebone.toolbar.rememberCollapsingToolbarScaffoldState
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.CollapsingToolbarScaffold
import net.mullvad.mullvadvpn.compose.component.CollapsingTopBar
import net.mullvad.mullvadvpn.compose.dialog.ReportProblemNoEmailDialog
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
            uiState = ReportProblemUiState(false, SendingReportUiState.Success("email@mail.com"))
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
            Column(
                modifier =
                    Modifier.fillMaxSize()
                        .padding(vertical = Dimens.mediumPadding, horizontal = Dimens.sideMargin)
            ) {
                when (uiState.sendingState) {
                    SendingReportUiState.Sending -> SendingContent()
                    is SendingReportUiState.Error ->
                        ErrorContent({ onSendReport(email, description) }, onClearSendResult)
                    is SendingReportUiState.Success -> SentContent(uiState.sendingState)
                }
                //            ShowReportProblemState(
                //                uiState.sendingState,
                //                onDismiss = onClearSendResult,
                //                onClearForm = {
                //                    email = ""
                //                    description = ""
                //                },
                //                retry = { onSendReport(email, description) }
                //            )
                return@CollapsingToolbarScaffold
            }
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

@Composable
private fun ColumnScope.SendingContent() {
    CircularProgressIndicator(
        modifier = Modifier.align(Alignment.CenterHorizontally),
        strokeCap = StrokeCap.Round,
        strokeWidth = Dimens.loadingSpinnerStrokeWidth
    )
    Spacer(modifier = Modifier.height(Dimens.problemReportIconToTitlePadding))
    Text(
        text = stringResource(id = R.string.sending),
        style = MaterialTheme.typography.headlineLarge,
        color = MaterialTheme.colorScheme.onBackground
    )
}

@Composable
private fun ColumnScope.SentContent(sendingState: SendingReportUiState.Success) {
    Icon(
        painter = painterResource(id = R.drawable.icon_success),
        contentDescription = stringResource(id = R.string.sent),
        modifier = Modifier.align(Alignment.CenterHorizontally).size(Dimens.dialogIconHeight),
        tint = Color.Unspecified
    )

    Spacer(modifier = Modifier.height(Dimens.problemReportIconToTitlePadding))
    Text(
        text = stringResource(id = R.string.sent),
        style = MaterialTheme.typography.headlineLarge,
        color = MaterialTheme.colorScheme.onBackground
    )
    Text(
        text =
            buildAnnotatedString {
                withStyle(SpanStyle(color = MaterialTheme.colorScheme.surface)) {
                    append(stringResource(id = R.string.sent_thanks))
                }
                append(" ")
                withStyle(SpanStyle(color = MaterialTheme.colorScheme.onPrimary)) {
                    append(stringResource(id = R.string.we_will_look_into_this))
                }
            },
        style = MaterialTheme.typography.bodySmall,
        modifier = Modifier.fillMaxWidth()
    )

    Spacer(modifier = Modifier.height(Dimens.smallPadding))
    sendingState.email?.let {
        val emailTemplate = stringResource(R.string.sent_contact)
        val annotatedEmailString =
            remember(it) {
                val emailStart = emailTemplate.indexOf('%')

                buildAnnotatedString {
                    append(emailTemplate.substring(0, emailStart))
                    withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
                        append(sendingState.email)
                    }
                }
            }

        Text(
            text = annotatedEmailString,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onBackground,
            modifier = Modifier.fillMaxWidth()
        )
    }
}

@Composable
private fun ColumnScope.ErrorContent(retry: () -> Unit, onDismiss: () -> Unit) {
    Icon(
        painter = painterResource(id = R.drawable.icon_fail),
        contentDescription = stringResource(id = R.string.failed_to_send),
        modifier = Modifier.size(Dimens.dialogIconHeight).align(Alignment.CenterHorizontally),
        tint = Color.Unspecified
    )
    Spacer(modifier = Modifier.height(Dimens.problemReportIconToTitlePadding))
    Text(
        text = stringResource(id = R.string.failed_to_send),
        style = MaterialTheme.typography.headlineLarge,
        color = MaterialTheme.colorScheme.onBackground,
    )
    Text(
        text = stringResource(id = R.string.failed_to_send_details),
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onBackground,
        modifier = Modifier.fillMaxWidth()
    )
    Spacer(modifier = Modifier.weight(1f))
    ActionButton(
        modifier = Modifier.fillMaxWidth().padding(vertical = Dimens.mediumPadding),
        colors =
            ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.primary,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
        onClick = onDismiss,
        text = stringResource(id = R.string.edit_message)
    )
    ActionButton(
        modifier = Modifier.fillMaxWidth(),
        colors =
            ButtonDefaults.buttonColors(
                containerColor = MaterialTheme.colorScheme.surface,
                contentColor = MaterialTheme.colorScheme.onPrimary,
            ),
        onClick = retry,
        text = stringResource(id = R.string.try_again)
    )
}
