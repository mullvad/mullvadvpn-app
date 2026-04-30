package net.mullvad.mullvadvpn.feature.problemreport.impl

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
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
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.SecureScreenWhileInView
import net.mullvad.mullvadvpn.common.compose.clickableAnnotatedString
import net.mullvad.mullvadvpn.common.compose.createUriHook
import net.mullvad.mullvadvpn.common.compose.isTv
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNoEmailConfirmedNavResult
import net.mullvad.mullvadvpn.feature.problemreport.api.ProblemReportNoEmailNavKey
import net.mullvad.mullvadvpn.feature.problemreport.api.ViewLogsNavKey
import net.mullvad.mullvadvpn.lib.common.util.appendHideNavOnPlayBuild
import net.mullvad.mullvadvpn.lib.ui.component.CheckboxConfirmation
import net.mullvad.mullvadvpn.lib.ui.component.ExpandChevron
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.ErrorSupportingText
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.Alpha40
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.positive
import net.mullvad.mullvadvpn.lib.ui.theme.color.warning
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

@Composable
fun ReportProblem(navigator: Navigator) {
    val vm = koinViewModel<ReportProblemViewModel>()
    val state by vm.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            is ReportProblemSideEffect.ShowConfirmNoEmail ->
                navigator.navigate(ProblemReportNoEmailNavKey)
        }
    }

    LocalResultStore.current.consumeResult<ProblemReportNoEmailConfirmedNavResult> {
        vm.sendReport(state.email, state.description, true)
    }

    ReportProblemScreen(
        state = state,
        onSendReport = { vm.sendReport(state.email, state.description) },
        onClearSendResult = vm::clearSendResult,
        onNavigateToViewLogs = dropUnlessResumed { navigator.navigate(ViewLogsNavKey) },
        onEmailChanged = vm::updateEmail,
        onDescriptionChanged = vm::updateDescription,
        onIncludeAccountIdCheckChange = vm::onIncludeAccountIdCheckChange,
        toggleShowIncludeAccountInformationWarningMessage =
            vm::showIncludeAccountInformationWarningMessage,
        onBackClick = dropUnlessResumed { navigator.goBack() },
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

    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.report_a_problem),
        navigationIcon = { unlessIsDetail { NavigateBackIconButton(onNavigateBack = onBackClick) } },
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
            val scrollState = rememberScrollState()
            Column(
                modifier =
                    Modifier
                        .imePadding() // imePadding needs to be applied before the parent modifier.
                        .then(modifier)
                        .drawVerticalScrollbar(
                            state = scrollState,
                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                        )
                        .verticalScroll(state = scrollState)
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
        textStyle = MaterialTheme.typography.bodyLarge.copy(textDirection = TextDirection.Ltr),
        placeholder = { Text(text = stringResource(id = R.string.user_email_hint)) },
        colors = mullvadDarkTextFieldColors(),
        keyboardOptions =
            KeyboardOptions(
                autoCorrectEnabled = false,
                keyboardType = KeyboardType.Email,
                imeAction = ImeAction.Next,
            ),
    )

    ProblemMessageTextField(
        value = state.description,
        isError = state.descriptionError != null,
        onDescriptionChanged = onDescriptionChanged,
    )

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
            isEnabled = state.logCollectingState == LogCollectingState.Success,
            isLoading = state.logCollectingState == LogCollectingState.Loading,
        )
        Spacer(modifier = Modifier.height(Dimens.buttonSpacing))
        VariantButton(onClick = onSendReport, text = stringResource(id = R.string.send))
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
    CheckboxConfirmation(
        text = stringResource(R.string.include_account_token_checkbox_text),
        includeAccountInformation,
        onIncludeAccountInformationCheckChange,
    ) {
        AccountInformationWarning(
            showIncludeAccountInformationWarningMessage =
                showIncludeAccountInformationWarningMessage,
            toggleShowIncludeAccountInformationWarningMessage =
                toggleShowIncludeAccountInformationWarningMessage,
            openPrivacyPolicy = openPrivacyPolicy,
        )
    }
}

@Composable
private fun AccountInformationWarning(
    showIncludeAccountInformationWarningMessage: Boolean,
    toggleShowIncludeAccountInformationWarningMessage: (Boolean) -> Unit,
    openPrivacyPolicy: () -> Unit,
) {
    Column(
        modifier =
            Modifier.padding(horizontal = Dimens.tinyPadding)
                .background(
                    color = MaterialTheme.colorScheme.tertiaryContainer.copy(alpha = Alpha40),
                    shape = MaterialTheme.shapes.medium,
                )
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
                            if (showIncludeAccountInformationWarningMessage) Dimens.tinyPadding
                            else Dimens.smallPadding,
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
                        onClick = { openPrivacyPolicy() },
                    ),
            )
        }
    }
}

@Composable
private fun ProblemMessageTextField(
    modifier: Modifier = Modifier,
    value: String,
    isError: Boolean,
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
        isError = isError,
        supportingText =
            if (isError) {
                { ErrorSupportingText(stringResource(R.string.report_problem_message_is_empty)) }
            } else null,
        colors = mullvadDarkTextFieldColors(),
        keyboardOptions =
            KeyboardOptions(
                capitalization = KeyboardCapitalization.Sentences,
                keyboardType = KeyboardType.Text,
                imeAction = if (isTv()) ImeAction.Next else ImeAction.Unspecified,
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
                withStyle(SpanStyle(color = MaterialTheme.colorScheme.positive)) {
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
