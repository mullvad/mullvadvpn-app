package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ErrorOutline
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.ReportProblemNoEmailDestination
import com.ramcosta.composedestinations.generated.destinations.ViewLogsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultRecipient
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.cell.CheckboxCell
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithMediumTopBar
import net.mullvad.mullvadvpn.compose.extensions.createUriHook
import net.mullvad.mullvadvpn.compose.preview.ReportProblemUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.SecureScreenWhileInView
import net.mullvad.mullvadvpn.compose.util.clickableAnnotatedString
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.warning
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.viewmodel.ReportProblemSideEffect
import net.mullvad.mullvadvpn.viewmodel.ReportProblemUiState
import net.mullvad.mullvadvpn.viewmodel.ReportProblemViewModel
import net.mullvad.mullvadvpn.viewmodel.SendingReportUiState
import org.koin.androidx.compose.koinViewModel

@Preview("Default|IncludeAccountNumber|ShowWarning|Sending|Success|Error")
@Composable
private fun PreviewReportProblemScreen(
    @PreviewParameter(ReportProblemUiStatePreviewParameterProvider::class)
    state: ReportProblemUiState
) {
    AppTheme {
        ReportProblemScreen(
            state = state,
            onSendReport = {},
            onClearSendResult = {},
            onNavigateToViewLogs = {},
            onEmailChanged = {},
            onDescriptionChanged = {},
            onIncludeAccountIdCheckChange = {},
            toggleShowIncludeAccountInformationWarningMessage = {},
            onBackClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun ReportProblem(
    navigator: DestinationsNavigator,
    noEmailConfirmResultRecipent: ResultRecipient<ReportProblemNoEmailDestination, Boolean>,
) {
    val vm = koinViewModel<ReportProblemViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            is ReportProblemSideEffect.ShowConfirmNoEmail ->
                navigator.navigate(ReportProblemNoEmailDestination)
        }
    }

    noEmailConfirmResultRecipent.onNavResult {
        when (it) {
            NavResult.Canceled -> {}
            is NavResult.Value -> vm.sendReport(state.email, state.description, true)
        }
    }

    ReportProblemScreen(
        state = state,
        onSendReport = { vm.sendReport(state.email, state.description) },
        onClearSendResult = vm::clearSendResult,
        onNavigateToViewLogs =
            dropUnlessResumed {
                navigator.navigate(ViewLogsDestination()) { launchSingleTop = true }
            },
        onEmailChanged = vm::updateEmail,
        onDescriptionChanged = vm::updateDescription,
        onIncludeAccountIdCheckChange = vm::onIncludeAccountIdCheckChange,
        toggleShowIncludeAccountInformationWarningMessage =
            vm::showIncludeAccountInformationWarningMessage,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
    )
}

@Composable
private fun ReportProblemScreen(
    state: ReportProblemUiState,
    onSendReport: () -> Unit,
    onClearSendResult: () -> Unit,
    onNavigateToViewLogs: () -> Unit,
    onEmailChanged: (String) -> Unit,
    onDescriptionChanged: (String) -> Unit,
    onIncludeAccountIdCheckChange: (Boolean) -> Unit,
    toggleShowIncludeAccountInformationWarningMessage: (Boolean) -> Unit,
    onBackClick: () -> Unit,
) {

    ScaffoldWithMediumTopBar(
        appBarTitle = stringResource(id = R.string.report_a_problem),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        // Show sending states
        if (state.sendingState != null) {
            Column(
                modifier =
                    modifier.padding(
                        vertical = Dimens.mediumPadding,
                        horizontal = Dimens.sideMargin,
                    )
            ) {
                when (state.sendingState) {
                    SendingReportUiState.Sending -> SendingContent()
                    is SendingReportUiState.Error -> ErrorContent(onSendReport, onClearSendResult)
                    is SendingReportUiState.Success -> SentContent(state.sendingState)
                }
            }
        } else {
            Column(
                modifier =
                    Modifier
                        .imePadding() // imePadding needs to be applied before the parent modifier.
                        .then(modifier)
                        .padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            bottom = Dimens.screenBottomMargin,
                        )
                        .height(IntrinsicSize.Max)
                        .animateContentSize(),
                verticalArrangement = Arrangement.spacedBy(Dimens.mediumPadding),
            ) {
                InputContent(
                    state = state,
                    onEmailChanged = onEmailChanged,
                    onDescriptionChanged = onDescriptionChanged,
                    onIncludeAccountIdCheckChange = onIncludeAccountIdCheckChange,
                    toggleShowIncludeAccountInformationWarningMessage =
                        toggleShowIncludeAccountInformationWarningMessage,
                    onNavigateToViewLogs = onNavigateToViewLogs,
                    onSendReport = onSendReport,
                )
            }
        }
    }
}

@Composable
private fun InputContent(
    state: ReportProblemUiState,
    onEmailChanged: (String) -> Unit,
    onDescriptionChanged: (String) -> Unit,
    onIncludeAccountIdCheckChange: (Boolean) -> Unit,
    toggleShowIncludeAccountInformationWarningMessage: (Boolean) -> Unit,
    onNavigateToViewLogs: () -> Unit,
    onSendReport: () -> Unit,
) {
    Description()

    TextField(
        modifier = Modifier.fillMaxWidth(),
        value = state.email,
        onValueChange = onEmailChanged,
        maxLines = 1,
        singleLine = true,
        placeholder = { Text(text = stringResource(id = R.string.user_email_hint)) },
        colors = mullvadWhiteTextFieldColors(),
        keyboardOptions =
            KeyboardOptions(
                autoCorrectEnabled = false,
                keyboardType = KeyboardType.Email,
                imeAction = ImeAction.Next,
            ),
    )

    ProblemMessageTextField(value = state.description, onDescriptionChanged = onDescriptionChanged)

    if (state.showIncludeAccountId) {
        IncludeAccountInformationCheckBox(
            includeAccountInformation = state.includeAccountId,
            onIncludeAccountInformationCheckChange = onIncludeAccountIdCheckChange,
            toggleShowIncludeAccountInformationWarningMessage =
                toggleShowIncludeAccountInformationWarningMessage,
            showIncludeAccountInformationWarningMessage = state.showIncludeAccountWarningMessage,
            isPlayBuild = state.isPlayBuild,
        )
    }

    Column {
        PrimaryButton(
            onClick = onNavigateToViewLogs,
            text = stringResource(id = R.string.view_logs),
        )
        Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
        VariantButton(
            onClick = onSendReport,
            isEnabled = state.description.isNotEmpty(),
            text = stringResource(id = R.string.send),
        )
    }
}

@Composable
private fun Description() {
    Column {
        Text(
            text = stringResource(id = R.string.problem_report_description),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelLarge,
        )
    }
}

@Composable
private fun IncludeAccountInformationCheckBox(
    includeAccountInformation: Boolean,
    showIncludeAccountInformationWarningMessage: Boolean,
    onIncludeAccountInformationCheckChange: (Boolean) -> Unit,
    toggleShowIncludeAccountInformationWarningMessage: (Boolean) -> Unit,
    isPlayBuild: Boolean,
) {
    val openPrivacyPolicy =
        LocalUriHandler.current.createUriHook(
            stringResource(R.string.privacy_policy_url).appendHideNavOnPlayBuild(isPlayBuild)
        )
    Column(
        modifier =
            Modifier.animateContentSize()
                .border(width = Dp.Hairline, color = MaterialTheme.colorScheme.primary)
                .padding(
                    bottom =
                        if (includeAccountInformation) {
                            Dimens.smallPadding
                        } else {
                            0.dp
                        }
                )
    ) {
        CheckboxCell(
            title = stringResource(R.string.include_account_token_checkbox_text),
            checked = includeAccountInformation,
            background = MaterialTheme.colorScheme.surface,
            startPadding = 0.dp,
            textStyle = MaterialTheme.typography.bodyMedium,
            onCheckedChange = onIncludeAccountInformationCheckChange,
        )
        if (includeAccountInformation) {
            AccountInformationWarning(
                showIncludeAccountInformationWarningMessage =
                    showIncludeAccountInformationWarningMessage,
                toggleShowIncludeAccountInformationWarningMessage =
                    toggleShowIncludeAccountInformationWarningMessage,
                openPrivacyPolicy = { openPrivacyPolicy() },
            )
        }
    }
}

@Composable
private fun AccountInformationWarning(
    showIncludeAccountInformationWarningMessage: Boolean,
    toggleShowIncludeAccountInformationWarningMessage: (Boolean) -> Unit,
    openPrivacyPolicy: (String) -> Unit,
) {
    Column(
        modifier =
            Modifier.padding(horizontal = Dimens.tinyPadding)
                .background(MaterialTheme.colorScheme.surfaceDim)
                .animateContentSize()
    ) {
        Row(
            modifier =
                Modifier.fillMaxWidth()
                    .clickable(
                        onClick = {
                            toggleShowIncludeAccountInformationWarningMessage(
                                !showIncludeAccountInformationWarningMessage
                            )
                        }
                    )
                    .padding(
                        top = Dimens.smallPadding,
                        start = Dimens.smallPadding,
                        end = Dimens.smallPadding,
                        bottom =
                            if (showIncludeAccountInformationWarningMessage) {
                                Dimens.tinyPadding
                            } else {
                                Dimens.smallPadding
                            },
                    )
        ) {
            Icon(
                imageVector = Icons.Outlined.ErrorOutline,
                contentDescription = stringResource(R.string.include_account_token_warning_title),
                tint = MaterialTheme.colorScheme.warning,
            )
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.smallPadding)
                        .weight(1f)
                        .align(Alignment.CenterVertically),
                style = MaterialTheme.typography.labelLarge,
                text = stringResource(R.string.include_account_token_warning_title),
            )
            ExpandChevron(isExpanded = showIncludeAccountInformationWarningMessage)
        }
        if (showIncludeAccountInformationWarningMessage) {
            Text(
                modifier =
                    Modifier.padding(horizontal = Dimens.smallPadding)
                        .padding(bottom = Dimens.smallPadding),
                style = MaterialTheme.typography.bodySmall,
                text =
                    clickableAnnotatedString(
                        text =
                            buildString {
                                appendLine(
                                    stringResource(
                                        R.string.include_account_token_warning_message_first
                                    )
                                )
                                append(
                                    stringResource(
                                        R.string.include_account_token_warning_message_second
                                    )
                                )
                            },
                        argument = stringResource(R.string.privacy_policy_lower_case),
                        linkStyle =
                            SpanStyle(
                                color = MaterialTheme.colorScheme.onSurface,
                                textDecoration = TextDecoration.Underline,
                            ),
                        onClick = openPrivacyPolicy,
                    ),
            )
        }
    }
}

@Composable
private fun ProblemMessageTextField(
    modifier: Modifier = Modifier,
    value: String,
    onDescriptionChanged: (String) -> Unit,
) {

    TextField(
        modifier =
            modifier
                .fillMaxWidth()
                .defaultMinSize(minHeight = Dimens.problemReportTextFieldMinHeight),
        value = value,
        onValueChange = onDescriptionChanged,
        placeholder = { Text(stringResource(R.string.user_message_hint)) },
        colors = mullvadWhiteTextFieldColors(),
        keyboardOptions =
            KeyboardOptions(
                capitalization = KeyboardCapitalization.Sentences,
                keyboardType = KeyboardType.Text,
                imeAction = ImeAction.Next,
            ),
    )
}

@Composable
private fun ColumnScope.SendingContent() {
    MullvadCircularProgressIndicatorLarge(modifier = Modifier.align(Alignment.CenterHorizontally))
    Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
    Text(
        text = stringResource(id = R.string.sending),
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
    )
}

@Composable
private fun ColumnScope.SentContent(sendingState: SendingReportUiState.Success) {
    SecureScreenWhileInView()
    Icon(
        painter = painterResource(id = R.drawable.icon_success),
        contentDescription = stringResource(id = R.string.sent),
        modifier = Modifier.align(Alignment.CenterHorizontally),
        tint = Color.Unspecified,
    )

    Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
    Text(
        text = stringResource(id = R.string.sent),
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
    )
    Text(
        text =
            buildAnnotatedString {
                withStyle(SpanStyle(color = MaterialTheme.colorScheme.tertiary)) {
                    append(stringResource(id = R.string.sent_thanks))
                }
                append(" ")
                withStyle(SpanStyle(color = MaterialTheme.colorScheme.onSurface)) {
                    append(stringResource(id = R.string.we_will_look_into_this))
                }
            },
        style = MaterialTheme.typography.bodyMedium,
        modifier = Modifier.fillMaxWidth(),
    )

    Spacer(modifier = Modifier.height(Dimens.smallPadding))
    sendingState.email?.let {
        val emailTemplate = stringResource(R.string.sent_contact)
        val annotatedEmailString =
            remember(it) {
                val emailStart = emailTemplate.indexOf('%')

                buildAnnotatedString {
                    append(emailTemplate.take(emailStart))
                    withStyle(SpanStyle(fontWeight = FontWeight.Bold)) {
                        append(sendingState.email)
                    }
                }
            }

        Text(
            text = annotatedEmailString,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}

@Composable
private fun ColumnScope.ErrorContent(retry: () -> Unit, onDismiss: () -> Unit) {
    Icon(
        painter = painterResource(id = R.drawable.icon_fail),
        contentDescription = stringResource(id = R.string.failed_to_send),
        modifier = Modifier.align(Alignment.CenterHorizontally),
        tint = Color.Unspecified,
    )
    Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
    Text(
        text = stringResource(id = R.string.failed_to_send),
        style = MaterialTheme.typography.headlineSmall,
        color = MaterialTheme.colorScheme.onSurface,
    )
    Text(
        text = stringResource(id = R.string.failed_to_send_details),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.fillMaxWidth(),
    )
    Spacer(modifier = Modifier.weight(1f))
    PrimaryButton(
        modifier =
            Modifier.fillMaxWidth()
                .padding(top = Dimens.mediumPadding, bottom = Dimens.buttonSpacing),
        onClick = onDismiss,
        text = stringResource(id = R.string.edit_message),
    )
    VariantButton(
        modifier = Modifier.fillMaxWidth(),
        onClick = retry,
        text = stringResource(id = R.string.try_again),
    )
}
