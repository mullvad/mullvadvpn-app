package net.mullvad.mullvadvpn.feature.deleteaccount.impl.deleteaccountconfirmation

import android.content.res.Resources
import androidx.activity.compose.BackHandler
import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.InputTransformation
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.byValue
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.rounded.Error
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.autofill.ContentDataType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.pluralStringResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentDataType
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.tooling.preview.Preview
import androidx.core.text.isDigitsOnly
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.collectLatest
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.accountNumberKeyboardType
import net.mullvad.mullvadvpn.common.compose.accountNumberOutputTransformation
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.deleteaccount.api.DeleteAccountCompleteNavKey
import net.mullvad.mullvadvpn.lib.model.DeleteAccountError
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorSmall
import net.mullvad.mullvadvpn.lib.ui.designsystem.NegativeButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.lib.ui.theme.color.warning
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Preview("Loading|Supported|Unsupported")
@Composable
private fun PreviewDeleteAccountConfirmation() {
    AppTheme {
        DeleteAccountConfirmation(
            state = DeleteAccountConfirmationUiState(daysLeft = DaysLeftState.DaysLeft(12)),
            onAccountInputChanged = {},
            deleteAccount = {},
            onBackClick = {},
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeleteAccountConfirmation(navigator: Navigator) {
    val vm = koinViewModel<DeleteAccountConfirmationViewModel>()
    val uiState = vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    if (uiState.value.isLoading) {
        // Consume back
        BackHandler {}
    }

    val resources = LocalResources.current
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            DeleteAccountConfirmationUiSideEffect.NavigateToComplete ->
                navigator.navigate(DeleteAccountCompleteNavKey)

            is DeleteAccountConfirmationUiSideEffect.DeleteAccountFailed ->
                snackbarHostState.showSnackbarImmediately(
                    it.deleteAccountError.toErrorMessage(resources)
                )
        }
    }
    DeleteAccountConfirmation(
        state = uiState.value,
        snackbarHostState = snackbarHostState,
        deleteAccount = vm::deleteAccount,
        onAccountInputChanged = vm::onAccountInputChanged,
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

private fun DeleteAccountError.toErrorMessage(resources: Resources): String =
    when (this) {
        is DeleteAccountError.UnableToReachApi -> resources.getString(R.string.unable_to_reach_api)
        is DeleteAccountError.Unknown,
        DeleteAccountError.AccountNumberDoesNotMatch ->
            resources.getString(R.string.delete_account_error)
    }

@ExperimentalMaterial3Api
@Composable
fun DeleteAccountConfirmation(
    state: DeleteAccountConfirmationUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onAccountInputChanged: (String) -> Unit,
    deleteAccount: () -> Unit,
    onBackClick: () -> Unit,
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.delete_account),
        navigationIcon = {
            NavigateBackIconButton(onNavigateBack = onBackClick, enabled = !state.isLoading)
        },
        snackbarHostState = snackbarHostState,
    ) { modifier ->
        DeleteAccountConfirmationContent(
            modifier,
            state,
            onAccountInputChanged,
            deleteAccount,
            onClickCancel = onBackClick,
        )
    }
}

@Composable
private fun DeleteAccountConfirmationContent(
    modifier: Modifier,
    state: DeleteAccountConfirmationUiState,
    onAccountInputChanged: (String) -> Unit,
    onClickDeleteAccount: () -> Unit,
    onClickCancel: () -> Unit,
) {
    val scrollState = rememberScrollState()
    Column(
        horizontalAlignment = Alignment.CenterHorizontally,
        modifier =
            modifier
                .drawVerticalScrollbar(
                    state = scrollState,
                    color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                )
                .verticalScroll(state = scrollState)
                .animateContentSize()
                .padding(horizontal = Dimens.sideMarginNew)
                .imePadding(),
    ) {
        Text(
            modifier = Modifier.fillMaxWidth(),
            text =
                buildAnnotatedString {
                    append(annotatedStringResource(R.string.delete_account_warning))
                    val bulletItems =
                        listOf(
                                R.string.delete_account_confirm_first_item,
                                R.string.delete_account_confirm_second_item,
                                R.string.delete_account_confirm_third_item,
                            )
                            .map { annotatedStringResource(it) }
                    withBulletList { bulletItems.forEach { withBulletListItem { append(it) } } }
                },
            style = MaterialTheme.typography.bodyLarge,
        )
        DaysLostWarning(state.daysLeft)
        Spacer(modifier = Modifier.height(Dimens.largePadding))
        Text(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(R.string.delete_account_confirmation_enter_account_number),
            style = MaterialTheme.typography.bodyLarge,
        )
        Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
        AccountNumberInput(onAccountInputChanged, state)
        Spacer(modifier = Modifier.height(Dimens.mediumSpacer))

        Spacer(Modifier.weight(1f))
        DeleteAccountConfirmationBottomBar(
            state.isLoading,
            onClickDeleteAccount = onClickDeleteAccount,
            onClickCancel = onClickCancel,
        )
    }
}

@Composable
private fun AccountNumberInput(
    onAccountInputChanged: (String) -> Unit,
    state: DeleteAccountConfirmationUiState,
) {
    val textFieldState = rememberTextFieldState()
    LaunchedEffect(textFieldState) {
        snapshotFlow { textFieldState.text.toString() }.collectLatest { onAccountInputChanged(it) }
    }

    var showLastChar by remember { mutableStateOf(false) }

    LaunchedEffect(textFieldState.text) {
        showLastChar = true
        delay(2.seconds)
        showLastChar = false
    }

    var showPassword by remember { mutableStateOf(false) }

    val transformation =
        remember(showPassword, showLastChar) {
            accountNumberOutputTransformation(
                showAccount = showPassword,
                showLastX = if (showLastChar) 1 else 0,
            )
        }

    val keyboardController = LocalSoftwareKeyboardController.current
    TextField(
        state = textFieldState,
        modifier = Modifier.semantics { contentDataType = ContentDataType.None }.fillMaxWidth(),
        trailingIcon = {
            IconButton(onClick = { showPassword = !showPassword }) {
                Icon(
                    imageVector =
                        if (showPassword) Icons.Outlined.VisibilityOff
                        else Icons.Outlined.Visibility,
                    contentDescription =
                        if (showPassword) stringResource(id = R.string.hide_account_number)
                        else stringResource(id = R.string.show_account_number),
                )
            }
        },
        placeholder = { Text(stringResource(R.string.account_delete_placeholder)) },
        keyboardOptions =
            KeyboardOptions(
                imeAction = ImeAction.Done,
                autoCorrectEnabled = false,
                keyboardType = KeyboardType.accountNumberKeyboardType(LocalContext.current),
            ),
        onKeyboardAction = { keyboardController?.hide() },
        inputTransformation =
            InputTransformation.byValue { current, proposed ->
                if (proposed.isDigitsOnly() && levenshtein(current, proposed) <= 1) proposed
                else current
            },
        outputTransformation = transformation,
        lineLimits = TextFieldLineLimits.SingleLine,
        enabled = !state.isLoading,
        colors = mullvadDarkTextFieldColors(),
        textStyle = MaterialTheme.typography.bodyLarge.copy(textDirection = TextDirection.Ltr),
        isError = state.verificationError != null,
        supportingText = state.verificationError?.let { { Text(it.toErrorMessage()) } },
    )
}

@Composable
private fun VerifyAccountError.toErrorMessage(): String =
    when (this) {
        VerifyAccountError.AccountDoesNotMatch ->
            stringResource(R.string.delete_account_number_must_match)
        VerifyAccountError.CouldNotFetchAccountNumber ->
            stringResource(R.string.delete_account_unable_to_verify)
    }

@Composable
internal fun DaysLostWarning(daysLeft: DaysLeftState) {
    if (daysLeft is DaysLeftState.NoDaysLeft) return

    Row(
        modifier = Modifier.fillMaxWidth().animateContentSize().padding(top = Dimens.mediumSpacer),
        horizontalArrangement = Arrangement.spacedBy(Dimens.tinyPadding),
    ) {
        Icon(
            imageVector = Icons.Rounded.Error,
            tint =
                when (daysLeft) {
                    is DaysLeftState.DaysLeft -> MaterialTheme.colorScheme.warning
                    DaysLeftState.Error -> MaterialTheme.colorScheme.error
                    DaysLeftState.Loading -> MaterialTheme.colorScheme.warning
                },
            contentDescription = null,
        )
        Text(
            text =
                when (daysLeft) {
                    is DaysLeftState.DaysLeft ->
                        pluralStringResource(
                            R.plurals.delete_account_days_left_warning,
                            daysLeft.value,
                            daysLeft.value,
                        )
                    DaysLeftState.Error ->
                        stringResource(R.string.delete_account_failed_loading_days)
                    DaysLeftState.Loading -> stringResource(R.string.delete_account_loading_days)
                },
            style = MaterialTheme.typography.bodyLarge,
        )
        if (daysLeft is DaysLeftState.Loading) {
            MullvadCircularProgressIndicatorSmall()
        }
    }
}

@Composable
private fun DeleteAccountConfirmationBottomBar(
    isLoading: Boolean,
    onClickDeleteAccount: () -> Unit,
    onClickCancel: () -> Unit,
) {
    Column(
        modifier =
            Modifier.padding(
                start = Dimens.smallPadding,
                end = Dimens.smallPadding,
                bottom = Dimens.screenBottomMargin,
            ),
        verticalArrangement = Arrangement.spacedBy(Dimens.smallPadding),
    ) {
        NegativeButton(
            text = stringResource(R.string.delete_account),
            onClick = onClickDeleteAccount,
            isEnabled = !isLoading,
            isLoading = isLoading,
        )
        PrimaryButton(
            onClick = onClickCancel,
            text = stringResource(R.string.cancel),
            isEnabled = !isLoading,
        )
    }
}
