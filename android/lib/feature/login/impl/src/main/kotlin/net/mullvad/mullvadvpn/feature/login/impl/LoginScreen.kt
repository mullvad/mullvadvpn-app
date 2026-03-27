package net.mullvad.mullvadvpn.feature.login.impl

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
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Visibility
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.icons.rounded.Clear
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.autofill.ContentType
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusProperties
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
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextDirection
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.compose.ui.unit.LayoutDirection
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.common.compose.ACCOUNT_NUMBER_CHUNK_SIZE
import net.mullvad.mullvadvpn.common.compose.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.common.compose.accountNumberKeyboardType
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
import net.mullvad.mullvadvpn.feature.login.impl.qrcode.QRCodeImage
import net.mullvad.mullvadvpn.feature.settings.api.SettingsNavKey
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.lib.ui.component.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.designsystem.PrimaryButton
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
private const val BOTTOM_SPACER_WEIGHT = 3f
private val LAST_CHAR_VISIBILITY_TIMEOUT = 2.seconds

@Composable
fun Login(
    navigator: Navigator,
    accountNumber: String? = null,
    vm: LoginViewModel = koinViewModel(),
) {
    val state by vm.uiState.collectAsStateWithLifecycle()

    // Login with argument, e.g when user comes from Too Many Devices screen
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
            dropUnlessResumed { error: LoginUiStateError ->
                navigator.navigate(ApiUnreachableNavKey(action = error.toLoginAction()))
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
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit,
) {
    ScaffoldWithTopBar(
        snackbarHostState = snackbarHostState,
        topBarColor = MaterialTheme.colorScheme.primary,
        iconTintColor = MaterialTheme.colorScheme.onPrimary,
        onSettingsClicked = onSettingsClick,
        enabled = state.loginState is LoginState.Idle,
        onAccountClicked = null,
    ) {
        val scrollState = rememberScrollState()
        Column(
            modifier =
                Modifier.padding(it)
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.primary)
                    .verticalScroll(scrollState)
        ) {
            Spacer(modifier = Modifier.weight(TOP_SPACER_WEIGHT))
            LoginIcon(
                state.loginState,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.largePadding),
            )
            LoginContent(
                state,
                onAccountNumberChange,
                onLoginClick,
                onDeleteHistoryClick,
                onShowApiUnreachableDialog,
            )
            Spacer(modifier = Modifier.weight(BOTTOM_SPACER_WEIGHT))
            CreateAccountPanel(
                onCreateAccountClick,
                isEnabled = state.loginState is LoginState.Idle,
            )
        }
    }
}

@Composable
private fun LoginContent(
    state: LoginUiState,
    onAccountNumberChange: (String) -> Unit,
    onLoginClick: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit,
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit,
) {
    Column(
        modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.sideMargin),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Text(
            text = state.loginState.title(),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.testTag(LOGIN_TITLE_TEST_TAG)
                    .fillMaxWidth()
                    .padding(bottom = Dimens.smallPadding),
        )

        LoginInput(
            state,
            onLoginClick,
            onAccountNumberChange,
            onDeleteHistoryClick,
            onShowApiUnreachableDialog,
        )

        Spacer(modifier = Modifier.size(Dimens.largePadding))
        VariantButton(
            isEnabled = state.loginButtonEnabled,
            onClick = { onLoginClick(state.accountNumberInput) },
            text = stringResource(id = R.string.log_in),
            modifier =
                Modifier.testTag(LOGIN_BUTTON_TEST_TAG).padding(bottom = Dimens.mediumPadding),
        )

        state.loginTicket?.let { ticket ->
            Spacer(modifier = Modifier.size(Dimens.mediumPadding))
            QRCodeImage(text = ticket.value)
            Spacer(modifier = Modifier.size(Dimens.mediumPadding))
        }
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
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit,
) {
    SupportingText(
        Modifier.padding(bottom = Dimens.smallPadding),
        onShowApiUnreachableDialog = onShowApiUnreachableDialog,
        state = state,
    )

    var showLastChar by remember { mutableStateOf(false) }
    LaunchedEffect(state.accountNumberInput) {
        showLastChar = true
        delay(LAST_CHAR_VISIBILITY_TIMEOUT)
        showLastChar = false
    }

    var showPassword by remember { mutableStateOf(false) }

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
        value = state.accountNumberInput,
        label = {
            Text(
                text = stringResource(id = R.string.login_description),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        },
        trailingIcon = {
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
        },
        keyboardActions = KeyboardActions(onDone = { onLoginClick(state.accountNumberInput) }),
        keyboardOptions =
            KeyboardOptions(
                autoCorrectEnabled = false,
                imeAction = if (state.loginButtonEnabled) ImeAction.Done else ImeAction.None,
                keyboardType = KeyboardType.accountNumberKeyboardType(LocalContext.current),
            ),
        onValueChange = onAccountNumberChange,
        singleLine = true,
        maxLines = 1,
        visualTransformation =
            accountNumberVisualTransformation(showPassword, if (showLastChar) 1 else 0),
        enabled = state.loginState is LoginState.Idle,
        colors = mullvadWhiteTextFieldColors(),
        textStyle = MaterialTheme.typography.bodyLarge.copy(textDirection = TextDirection.Ltr),
        isError = state.loginState.isError(),
    )

    AnimatedVisibility(
        visible = state.lastUsedAccount != null && state.loginState is LoginState.Idle
    ) {
        val token = state.lastUsedAccount?.value.orEmpty()
        val accountTransformation =
            remember(showPassword) {
                accountNumberVisualTransformation(
                    showPassword,
                    showLastX = ACCOUNT_NUMBER_CHUNK_SIZE,
                )
            }
        val transformedText =
            remember(token, accountTransformation) {
                accountTransformation.filter(AnnotatedString(token)).text
            }

        // Since content is number we should always do Ltr
        CompositionLocalProvider(LocalLayoutDirection provides LayoutDirection.Ltr) {
            AccountDropDownItem(
                accountNumber = transformedText.toString(),
                onClick = {
                    state.lastUsedAccount?.let {
                        onAccountNumberChange(it.value)
                        onLoginClick(it.value)
                    }
                },
                enabled = state.loginState is LoginState.Idle,
                onDeleteClick = onDeleteHistoryClick,
            )
        }
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
                        is LoginUiStateError.CreateAccountError ->
                            R.string.create_account_fail_title
                        null -> R.string.log_in
                    }
                is LoginState.Loading -> R.string.logging_in_title
                LoginState.Success -> R.string.logged_in_title
            }
    )

@Composable
private fun SupportingText(
    modifier: Modifier = Modifier,
    state: LoginUiState,
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit,
) {
    Text(
        modifier = modifier,
        text = state.loginState.supportingText(onShowApiUnreachableDialog) ?: AnnotatedString(""),
        style = MaterialTheme.typography.labelLarge,
        color =
            if (state.loginState.isError()) {
                MaterialTheme.colorScheme.error
            } else {
                MaterialTheme.colorScheme.onPrimary
            },
    )
}

@Composable
@Suppress("CyclomaticComplexMethod")
private fun LoginState.supportingText(
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit
): AnnotatedString? =
    when (this) {
        is LoginState.Idle if
            (loginUiStateError is LoginUiStateError.LoginError.ApiUnreachable ||
                loginUiStateError is LoginUiStateError.CreateAccountError.ApiUnreachable)
         -> apiUnreachableText(loginUiStateError, onShowApiUnreachableDialog)
        is LoginState.Idle -> {
            when (loginUiStateError) {
                LoginUiStateError.LoginError.InvalidCredentials -> R.string.login_fail_description
                is LoginUiStateError.LoginError.InvalidInput -> R.string.login_error_invalid_input
                LoginUiStateError.LoginError.NoInternetConnection,
                LoginUiStateError.CreateAccountError.NoInternetConnection ->
                    R.string.no_internet_connection
                LoginUiStateError.LoginError.ApiUnreachable,
                LoginUiStateError.CreateAccountError.ApiUnreachable -> R.string.api_unreachable
                LoginUiStateError.LoginError.TooManyAttempts,
                LoginUiStateError.CreateAccountError.TooManyAttempts ->
                    R.string.login_error_too_many_attempts
                is LoginUiStateError.LoginError.Unknown -> R.string.error_occurred
                LoginUiStateError.CreateAccountError.Unknown -> R.string.failed_to_create_account
                null -> null
            }?.toAnnotatedString()
        }
        is LoginState.Loading.CreatingAccount -> R.string.creating_new_account.toAnnotatedString()
        is LoginState.Loading.LoggingIn -> R.string.logging_in_description.toAnnotatedString()
        LoginState.Success -> R.string.logged_in_description.toAnnotatedString()
    }

@Composable
private fun apiUnreachableText(
    state: LoginUiStateError,
    onShowApiUnreachableDialog: (LoginUiStateError) -> Unit,
): AnnotatedString =
    clickableAnnotatedString(
        text = stringResource(R.string.login_error_api_unreachable),
        argument = stringResource(R.string.read_more_here),
        linkStyle =
            SpanStyle(
                color = MaterialTheme.colorScheme.onPrimary,
                textDecoration = TextDecoration.Underline,
            ),
        onClick = { onShowApiUnreachableDialog(state) },
    )

@Composable
private fun Int.toAnnotatedString(): AnnotatedString = AnnotatedString(stringResource(this))

@Composable
private fun AccountDropDownItem(
    modifier: Modifier = Modifier,
    accountNumber: String,
    enabled: Boolean,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit,
) {
    Row(
        modifier =
            modifier
                .clip(
                    MaterialTheme.shapes.medium.copy(
                        topStart = CornerSize(0f),
                        topEnd = CornerSize(0f),
                    )
                )
                .background(MaterialTheme.colorScheme.background)
                .height(IntrinsicSize.Min),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            modifier =
                Modifier.clickable(enabled = enabled, onClick = onClick)
                    .fillMaxHeight()
                    .weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
            contentAlignment = Alignment.CenterStart,
        ) {
            Text(
                text = accountNumber,
                overflow = TextOverflow.Clip,
                style = MaterialTheme.typography.bodyLarge,
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

@Composable
private fun CreateAccountPanel(onCreateAccountClick: () -> Unit, isEnabled: Boolean) {
    Column(
        Modifier.fillMaxWidth()
            .background(MaterialTheme.colorScheme.background)
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenBottomMargin)
    ) {
        Text(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            style = MaterialTheme.typography.bodyMedium,
            text = stringResource(id = R.string.dont_have_an_account),
            color = MaterialTheme.colorScheme.onBackground,
        )
        PrimaryButton(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.create_new_account),
            isEnabled = isEnabled,
            onClick = onCreateAccountClick,
        )
    }
}

private fun LoginUiStateError.toLoginAction(): LoginAction =
    when (this) {
        is LoginUiStateError.LoginError.ApiUnreachable -> LoginAction.LOGIN
        is LoginUiStateError.CreateAccountError.ApiUnreachable -> LoginAction.CREATE_ACCOUNT
        else -> throw IllegalArgumentException("Not an API unreachable error")
    }
