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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
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
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.destinations.ReportProblemNoEmailDialogDestination
import net.mullvad.mullvadvpn.compose.destinations.ViewLogsDestination
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ReportProblemSideEffect
import net.mullvad.mullvadvpn.viewmodel.ReportProblemUiState
import net.mullvad.mullvadvpn.viewmodel.ReportProblemViewModel
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewReportProblemScreen() {
    AppTheme { ReportProblemScreen(state = ReportProblemUiState()) }
}

@Preview
@Composable
private fun PreviewReportProblemSendingScreen() {
    AppTheme {
        ReportProblemScreen(
            state = ReportProblemUiState(sendingState = SendingReportUiState.Sending),
        )
    }
}

@Preview
@Composable
private fun PreviewReportProblemSuccessScreen() {
    AppTheme {
        ReportProblemScreen(
            state =
                ReportProblemUiState(sendingState = SendingReportUiState.Success("email@mail.com")),
        )
    }
}

@Preview
@Composable
private fun PreviewReportProblemErrorScreen() {
    AppTheme {
        ReportProblemScreen(
            state =
                ReportProblemUiState(
                    sendingState =
                        SendingReportUiState.Error(SendProblemReportResult.Error.CollectLog)
                )
        )
    }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun ReportProblem(
    navigator: DestinationsNavigator,
    noEmailConfirmResultRecipent: ResultRecipient<ReportProblemNoEmailDialogDestination, Boolean>
) {
    val vm = koinViewModel<ReportProblemViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) {
        vm.uiSideEffect.collect {
            when (it) {
                is ReportProblemSideEffect.ShowConfirmNoEmail -> {
                    navigator.navigate(ReportProblemNoEmailDialogDestination)
                }
            }
        }
    }

    noEmailConfirmResultRecipent.onNavResult {
        when (it) {
            NavResult.Canceled -> {}
            is NavResult.Value -> vm.sendReport(state.email, state.description, true)
        }
    }

    ReportProblemScreen(
        state,
        onSendReport = { vm.sendReport(state.email, state.description) },
        onClearSendResult = vm::clearSendResult,
        onNavigateToViewLogs = {
            navigator.navigate(ViewLogsDestination()) { launchSingleTop = true }
        },
        onEmailChanged = vm::updateEmail,
        onDescriptionChanged = vm::updateDescription,
        onBackClick = navigator::navigateUp,
    )
}

@Composable
private fun ReportProblemScreen(
    state: ReportProblemUiState,
    onSendReport: () -> Unit = {},
    onClearSendResult: () -> Unit = {},
    onNavigateToViewLogs: () -> Unit = {},
    onEmailChanged: (String) -> Unit = {},
    onDescriptionChanged: (String) -> Unit = {},
    onBackClick: () -> Unit = {}
) {

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.report_a_problem),
        navigationIcon = { NavigateBackIconButton(onBackClick) }
    ) { modifier ->
        // Show sending states
        if (state.sendingState != null) {
            Column(
                modifier =
                    modifier.padding(
                        vertical = Dimens.mediumPadding,
                        horizontal = Dimens.sideMargin
                    )
            ) {
                when (state.sendingState) {
                    SendingReportUiState.Sending -> SendingContent()
                    is SendingReportUiState.Error -> ErrorContent(onSendReport, onClearSendResult)
                    is SendingReportUiState.Success -> SentContent(state.sendingState)
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
                value = state.email,
                onValueChange = onEmailChanged,
                maxLines = 1,
                singleLine = true,
                placeholder = { Text(text = stringResource(id = R.string.user_email_hint)) },
                colors = mullvadWhiteTextFieldColors()
            )

            TextField(
                modifier = Modifier.fillMaxWidth().weight(1f),
                value = state.description,
                onValueChange = onDescriptionChanged,
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
                    onClick = onSendReport,
                    isEnabled = state.description.isNotEmpty(),
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
