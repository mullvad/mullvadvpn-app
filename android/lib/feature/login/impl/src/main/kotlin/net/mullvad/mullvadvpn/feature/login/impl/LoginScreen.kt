package net.mullvad.mullvadvpn.feature.login.impl

import android.content.res.Resources
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CornerSize
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.text.input.TextFieldLineLimits
import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.foundation.text.input.rememberTextFieldState
import androidx.compose.foundation.text.input.setTextAndPlaceCursorAtEnd
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.rounded.Clear
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldLabelPosition
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.autofill.ContentType
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.layout.AlignmentLine
import androidx.compose.ui.layout.FirstBaseline
import androidx.compose.ui.layout.layout
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.platform.LocalResources
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.contentType
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.rememberTextMeasurer
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.ACCOUNT_NUMBER_CHUNK_SIZE
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.accountNumberKeyboardType
import net.mullvad.mullvadvpn.common.compose.accountNumberOutputTransformation
import net.mullvad.mullvadvpn.common.compose.accountNumberVisualTransformation
import net.mullvad.mullvadvpn.common.compose.clickableAnnotatedString
import net.mullvad.mullvadvpn.common.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.showSnackbarImmediately
import net.mullvad.mullvadvpn.core.LocalResultStore
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.home.api.ConnectNavKey
import net.mullvad.mullvadvpn.feature.home.api.OutOfTimeNavKey
import net.mullvad.mullvadvpn.feature.home.api.WelcomeNavKey
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableInfoDialogResult
import net.mullvad.mullvadvpn.feature.login.api.ApiUnreachableNavKey
import net.mullvad.mullvadvpn.feature.login.api.CreateAccountConfirmationNavKey
import net.mullvad.mullvadvpn.feature.login.api.DeviceListNavKey
import net.mullvad.mullvadvpn.feature.login.api.LoginAction
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadDarkTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryTextButton
import net.mullvad.mullvadvpn.lib.ui.designsystem.VariantButton
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_INPUT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_REVEAL_INPUT_BUTTON_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_SCREEN_DELETE_ACCOUNT_HISTORY_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.LOGIN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import org.koin.androidx.compose.koinViewModel

@Preview("Default|Loading.LoggingIn|Loading.CreatingAccount|LoginError|Success")
@Composable
private fun PreviewLoginScreen(
    @PreviewParameter(LoginUiStatePreviewParameterProvider::class) state: LoginUiState
) {
    AppTheme {
        LoginScreen(
            state = state,
            snackbarHostState = SnackbarHostState(),
            onLoginClick = {},
            onCreateAccountClick = {},
            onDeleteHistoryClick = {},
            onAccountNumberChange = {},
            onSettingsClick = {},
            onShowApiUnreachableDialog = {},
        )
    }
}

private const val TOP_SPACER_WEIGHT = 1f
private const val BOTTOM_SPACER_WEIGHT = 1f

@Composable
fun Login(
    navigator: Navigator,
    accountNumber: String? = null,
    vm: LoginViewModel = koinViewModel(),
) {
    val state by vm.uiState.collectAsStateWithLifecycle()

    // Login with argument, e.g. when user comes from Too Many Devices screen
    LaunchedEffect(accountNumber) {
        if (accountNumber != null) {
            vm.onAccountNumberChange(accountNumber)
            vm.login(accountNumber)
        }
    }

    val resources = LocalResources.current
    val snackbarHostState = remember { SnackbarHostState() }
    val scope = rememberCoroutineScope()

    LocalResultStore.current.consumeResult<CreateAccountConfirmationDialogResult> { result ->
        if (result.confirmed) vm.onCreateAccountConfirmed()
    }

    LocalResultStore.current.consumeResult<ApiUnreachableInfoDialogResult> {
        when (it) {
            ApiUnreachableInfoDialogResult.Error ->
                scope.launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = resources.getString(R.string.error_occurred)
                    )
                }
            is ApiUnreachableInfoDialogResult.Success -> {
                when (it.arg.action) {
                    LoginAction.LOGIN -> vm.login(state.accountNumberInput)
                    LoginAction.CREATE_ACCOUNT -> vm.onCreateAccountClick()
                }
            }
        }
    }

    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            LoginUiSideEffect.NavigateToWelcome ->
                navigator.navigate(WelcomeNavKey, clearBackStack = true)
            is LoginUiSideEffect.NavigateToConnect ->
                navigator.navigate(ConnectNavKey, clearBackStack = true)
            is LoginUiSideEffect.TooManyDevices ->
                navigator.navigate(DeviceListNavKey(it.accountNumber))
            LoginUiSideEffect.NavigateToOutOfTime ->
                navigator.navigate(OutOfTimeNavKey, clearBackStack = true)
            LoginUiSideEffect.NavigateToCreateAccountConfirmation ->
                navigator.navigate(CreateAccountConfirmationNavKey)
            LoginUiSideEffect.GenericError ->
                snackbarHostState.showSnackbarImmediately(
                    message = resources.getString(R.string.error_occurred)
                )
            is LoginUiSideEffect.CreateAccount ->
                snackbarHostState.showCreateAccountSnackbar(it, resources) {
                    snackbarHostState.currentSnackbarData?.dismiss()
                    navigator.navigate(ApiUnreachableNavKey(LoginAction.CREATE_ACCOUNT))
                }
        }
    }
    LoginScreen(
        state = state,
        snackbarHostState = snackbarHostState,
        onLoginClick = vm::login,
        onCreateAccountClick = vm::onCreateAccountClick,
        onDeleteHistoryClick = vm::clearAccountHistory,
        onAccountNumberChange = vm::onAccountNumberChange,
        onSettingsClick = dropUnlessResumed { navigator.navigate(SettingsNavKey) },
        onShowApiUnreachableDialog =
            dropUnlessResumed { action: LoginAction ->
                navigator.navigate(ApiUnreachableNavKey(action = action))
            },
    )
}

@Composable
private fun LoginScreen(
    state: LoginUiState,
    snackbarHostState: SnackbarHostState = SnackbarHostState(),
    onLoginClick: (String) -> Unit,
    onCreateAccountClick: () -> Unit,
    onDeleteHistoryClick: () -> Unit,
    onAccountNumberChange: (String) -> Unit,
    onSettingsClick: () -> Unit,
    onShowApiUnreachableDialog: (LoginAction) -> Unit,
) {
    ScaffoldWithTopBar(
        snackbarHostState = snackbarHostState,
        topBarColor = MaterialTheme.colorScheme.background,
        iconTintColor = MaterialTheme.colorScheme.onBackground,
        onSettingsClicked = onSettingsClick,
        enabled = state.loginState is LoginState.Idle,
        onAccountClicked = null,
    ) {
        val scrollState = rememberScrollState()
        Column(
            modifier =
                Modifier.padding(it)
                    .padding(horizontal = Dimens.sideMargin)
                    .fillMaxSize()
                    .verticalScroll(scrollState)
        ) {
            Spacer(modifier = Modifier.weight(TOP_SPACER_WEIGHT))
            LoginIcon(
                state.loginState,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.largePadding),
            )
            Text(
                text = state.loginState.title(),
                style = MaterialTheme.typography.headlineLarge,
                color = MaterialTheme.colorScheme.onPrimary,
                modifier =
                    Modifier.testTag(LOGIN_TITLE_TEST_TAG)
                        .fillMaxWidth()
                        .padding(bottom = Dimens.smallPadding),
            )

            Column {
                LoginInput(
                    state,
                    onLoginClick,
                    onAccountNumberChange,
                    onDeleteHistoryClick,
                    onShowApiUnreachableDialog,
                )
            }

            AnimatedVisibility(state.loginState is LoginState.Idle) {
                Column {
                    Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
                    VariantButton(
                        isEnabled = state.loginButtonEnabled,
                        onClick = { onLoginClick(state.accountNumberInput) },
                        text = stringResource(id = R.string.log_in),
                        modifier = Modifier.testTag(LOGIN_BUTTON_TEST_TAG),
                    )
                    Spacer(modifier = Modifier.height(Dimens.largePadding))
                    OrDivier()
                    Spacer(modifier = Modifier.height(Dimens.mediumSpacer))
                    PrimaryTextButton(
                        modifier = Modifier.align(Alignment.CenterHorizontally),
                        text = stringResource(id = R.string.create_new_account),
                        isEnabled = state.loginState is LoginState.Idle,
                        onClick = onCreateAccountClick,
                    )
                }
            }
            Spacer(modifier = Modifier.weight(BOTTOM_SPACER_WEIGHT))
        }
    }
}

@Composable
fun OrDivier() {
    Row(verticalAlignment = Alignment.CenterVertically) {
        HorizontalDivider(
            modifier = Modifier.weight(1f),
            color = MaterialTheme.colorScheme.onBackground,
        )
        Text(
            "Or",
            modifier = Modifier.padding(horizontal = Dimens.smallPadding),
            color = MaterialTheme.colorScheme.onBackground,
        )
        HorizontalDivider(
            modifier = Modifier.weight(1f),
            color = MaterialTheme.colorScheme.onBackground,
        )
    }
}

@Composable
@Suppress("LongMethod")
@OptIn(ExperimentalComposeUiApi::class)
private fun ColumnScope.LoginInput(
    state: LoginUiState,
    onLoginClick: (String) -> Unit,
    onAccountNumberChange: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit,
    onShowApiUnreachableDialog: (LoginAction) -> Unit,
) {

    var showPassword by remember { mutableStateOf(false) }
    val outputTransformation =
        remember(showPassword) { accountNumberOutputTransformation(showPassword) }

    val accountState = rememberTextFieldState(state.accountNumberInput)
    LaunchedEffect(accountState) {
        snapshotFlow { accountState.text.toString() }.collectLatest { onAccountNumberChange(it) }
    }
    LaunchedEffect(accountState) {}
    TextField(
        modifier =
            // Fix for DPad navigation
            Modifier.semantics { contentType = ContentType.Password }
                .focusProperties {
                    left = FocusRequester.Cancel
                    right = FocusRequester.Cancel
                }
                .fillMaxWidth()
                .testTag(LOGIN_INPUT_TEST_TAG)
                .let {
                    if (state.lastUsedAccount == null) {
                        it.clip(MaterialTheme.shapes.small)
                    } else {
                        it
                    }
                },
        state =
            if (state.loginState is LoginState.Loading.CreatingAccount) TextFieldState("")
            else {
                accountState
            },
        labelPosition = TextFieldLabelPosition.Above(),
        label = {
            Text(
                text = stringResource(id = R.string.account_number),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        lineLimits = TextFieldLineLimits.SingleLine,
        trailingIcon =
            if (state.loginState is LoginState.Idle) {
                {
                    IconButton(
                        modifier = Modifier.testTag(LOGIN_REVEAL_INPUT_BUTTON_TEST_TAG),
                        onClick = { showPassword = !showPassword },
                    ) {
                        Icon(
                            imageVector =
                                if (showPassword) Icons.Outlined.VisibilityOff
                                else Icons.Outlined.Visibility,
                            contentDescription =
                                if (showPassword) stringResource(id = R.string.hide_account_number)
                                else stringResource(id = R.string.show_account_number),
                        )
                    }
                }
            } else null,
        placeholder = {
            if (state.loginState == LoginState.Loading.CreatingAccount) {
                Text(stringResource(R.string.generating_account_number))
            } else {
                Text(stringResource(R.string.login_description))
            }
        },
        onKeyboardAction = { onLoginClick(state.accountNumberInput) },
        keyboardOptions =
            KeyboardOptions(
                autoCorrectEnabled = false,
                imeAction = if (state.loginButtonEnabled) ImeAction.Done else ImeAction.None,
                keyboardType = KeyboardType.accountNumberKeyboardType(LocalContext.current),
            ),
        outputTransformation = outputTransformation,
        enabled = state.loginState is LoginState.Idle,
        textStyle =
            MaterialTheme.typography.bodyLarge.copy(
                textDirection = TextDirection.Ltr,
                fontFamily = FontFamily.Monospace,
            ),
        colors = mullvadDarkTextFieldColors(),
        isError = state.loginState.isError(),
    )

    AnimatedVisibility(
        visible = state.lastUsedAccount != null && state.loginState is LoginState.Idle
    ) {
        CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
            AccountDropDownItem(
                accountNumber = state.lastUsedAccount?.value.orEmpty(),
                showPassword = showPassword,
                onClick = {
                    state.lastUsedAccount?.let {
                        accountState.setTextAndPlaceCursorAtEnd(it.value)
                        onLoginClick(it.value)
                    }
                },
                enabled = state.loginState is LoginState.Idle,
                onDeleteClick = onDeleteHistoryClick,
            )
        }
    }

    val text = state.loginState.supportingText(onShowApiUnreachableDialog)
    AnimatedVisibility(text != null) {
        SupportingText(
            Modifier.padding(top = Dimens.tinyPadding),
            text = text ?: AnnotatedString(""),
        )
    }
}

@Composable
private fun LoginIcon(loginState: LoginState, modifier: Modifier = Modifier) {
    Box(contentAlignment = Alignment.Center, modifier = modifier) {
        when (loginState) {
            is LoginState.Idle ->
                if (loginState.loginUiStateError != null) {
                    Image(
                        painter = painterResource(id = R.drawable.icon_fail),
                        contentDescription = stringResource(id = R.string.login_fail_title),
                    )
                } else {
                    // If view is Idle, we display empty box to keep the same size as other states
                }
            is LoginState.Loading -> MullvadCircularProgressIndicatorLarge()
            LoginState.Success ->
                Image(
                    painter = painterResource(id = R.drawable.icon_success),
                    contentDescription = stringResource(id = R.string.logged_in_title),
                )
        }
    }
}

@Composable
private fun LoginState.title(): String =
    stringResource(
        id =
            when (this) {
                is LoginState.Idle ->
                    when (this.loginUiStateError) {
                        is LoginUiStateError.LoginError -> R.string.login_fail_title
                        null -> R.string.log_in
                    }
                is LoginState.Loading.LoggingIn -> R.string.logging_in_title
                is LoginState.Loading.CreatingAccount -> R.string.creating_new_account
                LoginState.Success -> R.string.logged_in_title
            }
    )

@Composable
private fun SupportingText(modifier: Modifier = Modifier, text: AnnotatedString) {
    Text(
        modifier = modifier,
        text = text,
        style = MaterialTheme.typography.labelLarge,
        color = MaterialTheme.colorScheme.error,
    )
}

@Composable
@Suppress("CyclomaticComplexMethod")
private fun LoginState.supportingText(
    onShowApiUnreachableDialog: (LoginAction) -> Unit
): AnnotatedString? =
    when (this) {
        is LoginState.Idle if loginUiStateError is LoginUiStateError.LoginError.ApiUnreachable ->
            apiUnreachableText(onShowApiUnreachableDialog)
        is LoginState.Idle -> {
            when (loginUiStateError) {
                LoginUiStateError.LoginError.InvalidCredentials,
                is LoginUiStateError.LoginError.InvalidInput -> R.string.login_fail_description
                LoginUiStateError.LoginError.NoInternetConnection -> R.string.no_internet_connection
                LoginUiStateError.LoginError.ApiUnreachable -> R.string.api_unreachable
                LoginUiStateError.LoginError.TooManyAttempts ->
                    R.string.login_error_too_many_attempts
                is LoginUiStateError.LoginError.Unknown -> R.string.error_occurred
                null -> null
                LoginUiStateError.LoginError.Empty -> R.string.login_fail_empty
            }?.toAnnotatedString()
        }
        is LoginState.Loading.CreatingAccount -> null
        is LoginState.Loading.LoggingIn -> null
        LoginState.Success -> null
    }

@Composable
private fun apiUnreachableText(onShowApiUnreachableDialog: (LoginAction) -> Unit): AnnotatedString =
    clickableAnnotatedString(
        text = stringResource(R.string.login_error_api_unreachable),
        argument = stringResource(R.string.read_more_here),
        linkStyle =
            SpanStyle(
                color = MaterialTheme.colorScheme.onPrimary,
                textDecoration = TextDecoration.Underline,
            ),
        onClick = { onShowApiUnreachableDialog(LoginAction.LOGIN) },
    )

@Composable
private fun Int.toAnnotatedString(): AnnotatedString = AnnotatedString(stringResource(this))

@Composable
private fun AccountDropDownItem(
    modifier: Modifier = Modifier,
    showPassword: Boolean,
    accountNumber: String,
    enabled: Boolean,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit,
) {
    val accountTransformation =
        remember(showPassword) {
            accountNumberVisualTransformation(showPassword, showLastX = ACCOUNT_NUMBER_CHUNK_SIZE)
        }
    val transformedText =
        remember(accountNumber, accountTransformation) {
            accountTransformation.filter(AnnotatedString(accountNumber)).text
        }

    Row(
        modifier =
            modifier
                .clip(
                    MaterialTheme.shapes.medium.copy(
                        topStart = CornerSize(0f),
                        topEnd = CornerSize(0f),
                    )
                )
                .background(MaterialTheme.colorScheme.secondaryContainer)
                .height(IntrinsicSize.Min),
        verticalAlignment = Alignment.CenterVertically,
    ) {

        // Hack, our PASSWORD_UNICODE dot char changes the baseline height, so this workaround
        // ensures we always place it at the same baseline
        val textStyle = MaterialTheme.typography.bodyLarge.copy(fontFamily = FontFamily.Monospace)
        // Measure the digit baseline once to use as a fixed reference
        val textMeasurer = rememberTextMeasurer()
        val digitBaseline =
            remember(textStyle) {
                textMeasurer
                    .measure(text = AnnotatedString("0"), style = textStyle, maxLines = 1)
                    .firstBaseline
            }

        Box(
            modifier =
                Modifier.clickable(enabled = enabled, onClick = onClick)
                    .fillMaxHeight()
                    .weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
            contentAlignment = Alignment.CenterStart,
        ) {
            Text(
                text = transformedText,
                overflow = TextOverflow.Clip,
                style = textStyle,
                maxLines = 1,
                // Place text according to baseline so text does not jump as user hide/show password
                modifier =
                    Modifier.layout { measurable, constraints ->
                        val placeable = measurable.measure(constraints)
                        val actualBaseline = placeable[FirstBaseline]
                        // Shift the text so its baseline aligns with the digit baseline
                        val yOffset =
                            if (actualBaseline != AlignmentLine.Unspecified) {
                                digitBaseline.toInt() - actualBaseline
                            } else {
                                0
                            }
                        layout(placeable.width, placeable.height) { placeable.place(0, yOffset) }
                    },
            )
        }
        IconButton(
            modifier = Modifier.testTag(LOGIN_SCREEN_DELETE_ACCOUNT_HISTORY_TEST_TAG),
            enabled = enabled,
            onClick = onDeleteClick,
        ) {
            Icon(
                imageVector = Icons.Rounded.Clear,
                contentDescription = null,
                modifier = Modifier.size(Dimens.listIconSize),
            )
        }
    }
}

private suspend fun SnackbarHostState.showCreateAccountSnackbar(
    effect: LoginUiSideEffect.CreateAccount,
    resources: Resources,
    onShowApiUnreachableDialog: () -> Unit,
) {
    val message =
        resources.getString(
            when (effect) {
                LoginUiSideEffect.CreateAccount.ApiUnreachable ->
                    R.string.unable_to_reach_api
                LoginUiSideEffect.CreateAccount.NoInternet -> R.string.no_internet_connection
                LoginUiSideEffect.CreateAccount.TimeOut -> R.string.login_fail_empty
                LoginUiSideEffect.CreateAccount.TooManyAttempts ->
                    R.string.login_error_too_many_attempts
                is LoginUiSideEffect.CreateAccount.Unknown -> R.string.failed_to_create_account
            }
        )
    showSnackbarImmediately(
        message = message,
        if (effect is LoginUiSideEffect.CreateAccount.ApiUnreachable)
            resources.getString(R.string.read_more)
        else null,
        onAction =
            if (effect is LoginUiSideEffect.CreateAccount.ApiUnreachable) {
                { onShowApiUnreachableDialog() }
            } else null,
        duration = SnackbarDuration.Short
    )
}
