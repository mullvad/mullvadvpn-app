package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.dialog.ReportProblemNoEmailDialog
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ReportProblemUiState
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState

@Preview
@Composable
private fun PreviewReportProblemScreen() {
    AppTheme { ReportProblemScreen(uiState = ReportProblemUiState()) }
}

@Preview
@Composable
private fun PreviewReportProblemSendingScreen() {
    AppTheme {
        ReportProblemScreen(uiState = ReportProblemUiState(false, SendingReportUiState.Sending))
    }
}

@Preview
@Composable
private fun PreviewReportProblemConfirmNoEmailScreen() {
    AppTheme { ReportProblemScreen(uiState = ReportProblemUiState(true)) }
}

@Preview
@Composable
private fun PreviewReportProblemSuccessScreen() {
    AppTheme {
        ReportProblemScreen(
            uiState = ReportProblemUiState(false, SendingReportUiState.Success("email@mail.com"))
        )
    }
}

@Preview
@Composable
private fun PreviewReportProblemErrorScreen() {
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
    updateEmail: (String) -> Unit = {},
    updateDescription: (String) -> Unit = {},
    onBackClick: () -> Unit = {}
) {
    // Dialog to show confirm if no email was added
    if (uiState.showConfirmNoEmail) {
        ReportProblemNoEmailDialog(
            onDismiss = onDismissNoEmailDialog,
            onConfirm = { onSendReport(uiState.email, uiState.description) }
        )
    }

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.report_a_problem),
        navigationIcon = { NavigateBackIconButton(onBackClick) }
    ) { modifier ->

        // Show sending states
        if (uiState.sendingState != null) {
            Column(
                modifier =
                    modifier.padding(
                        vertical = Dimens.mediumPadding,
                        horizontal = Dimens.sideMargin
                    )
            ) {
                when (uiState.sendingState) {
                    SendingReportUiState.Sending -> SendingContent()
                    is SendingReportUiState.Error ->
                        ErrorContent(
                            { onSendReport(uiState.email, uiState.description) },
                            onClearSendResult
                        )
                    is SendingReportUiState.Success -> SentContent(uiState.sendingState)
                }
                return@ScaffoldWithMediumTopBar
            }
        }

        Column(
            modifier =
                modifier
                    .padding(
                        start = Dimens.sideMargin,
                        end = Dimens.sideMargin,
                        bottom = Dimens.verticalSpace,
                    )
                    .height(IntrinsicSize.Max),
            verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding)
        ) {
            Text(text = stringResource(id = R.string.problem_report_description))

            TextField(
                modifier = Modifier.fillMaxWidth(),
                value = uiState.email,
                onValueChange = updateEmail,
                maxLines = 1,
                singleLine = true,
                placeholder = { Text(text = stringResource(id = R.string.user_email_hint)) },
                colors = mullvadWhiteTextFieldColors()
            )

            TextField(
                modifier = Modifier.fillMaxWidth().weight(1f),
                value = uiState.description,
                onValueChange = updateDescription,
                placeholder = { Text(stringResource(R.string.user_message_hint)) },
                colors = mullvadWhiteTextFieldColors()
            )

            Column {
                PrimaryButton(
                    onClick = onNavigateToViewLogs,
                    text = stringResource(id = R.string.view_logs)
                )
                Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
                VariantButton(
                    onClick = { onSendReport(uiState.email, uiState.description) },
                    isEnabled = uiState.description.isNotEmpty(),
                    text = stringResource(id = R.string.send)
                )
            }
        }
    }
}

@Composable
private fun ColumnScope.SendingContent() {
    MullvadCircularProgressIndicatorLarge(
        modifier = Modifier.align(Alignment.CenterHorizontally),
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
    SecureScreenWhileInView()
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
    PrimaryButton(
        modifier =
            Modifier.fillMaxWidth()
                .padding(top = Dimens.mediumPadding, bottom = Dimens.buttonSpacing),
        onClick = onDismiss,
        text = stringResource(id = R.string.edit_message)
    )
    VariantButton(
        modifier = Modifier.fillMaxWidth(),
        onClick = retry,
        text = stringResource(id = R.string.try_again)
    )
}
