package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.FocusState
import androidx.compose.ui.focus.focusProperties
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.button.VariantButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.*
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.compose.test.LOGIN_TITLE_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.mullvadWhiteTextFieldColors
import net.mullvad.mullvadvpn.compose.util.accountTokenVisualTransformation
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaTopBar

@Preview
@Composable
private fun PreviewIdle() {
    AppTheme { LoginScreen(uiState = LoginUiState()) }
}

@Preview
@Composable
private fun PreviewLoggingIn() {
    AppTheme { LoginScreen(uiState = LoginUiState(loginState = Loading.LoggingIn)) }
}

@Preview
@Composable
private fun PreviewCreatingAccount() {
    AppTheme { LoginScreen(uiState = LoginUiState(loginState = Loading.CreatingAccount)) }
}

@Preview
@Composable
private fun PreviewLoginError() {
    AppTheme {
        LoginScreen(uiState = LoginUiState(loginState = Idle(LoginError.InvalidCredentials)))
    }
}

@Preview
@Composable
private fun PreviewLoginSuccess() {
    AppTheme { LoginScreen(uiState = LoginUiState(loginState = Success)) }
}

@OptIn(ExperimentalComposeUiApi::class)
@Composable
fun LoginScreen(
    uiState: LoginUiState,
    onLoginClick: (String) -> Unit = {},
    onCreateAccountClick: () -> Unit = {},
    onDeleteHistoryClick: () -> Unit = {},
    onAccountNumberChange: (String) -> Unit = {},
    onSettingsClick: () -> Unit = {},
) {
    ScaffoldWithTopBar(
        modifier = Modifier.semantics { testTagsAsResourceId = true },
        topBarColor = MaterialTheme.colorScheme.primary,
        statusBarColor = MaterialTheme.colorScheme.primary,
        navigationBarColor = MaterialTheme.colorScheme.background,
        iconTintColor = MaterialTheme.colorScheme.onPrimary.copy(alpha = AlphaTopBar),
        onSettingsClicked = onSettingsClick,
        onAccountClicked = null
    ) {
        val scrollState = rememberScrollState()
        Column(
            modifier =
                Modifier.padding(it)
                    .fillMaxSize()
                    .background(MaterialTheme.colorScheme.primary)
                    .verticalScroll(scrollState)
        ) {
            Spacer(modifier = Modifier.weight(1f))
            LoginIcon(
                uiState.loginState,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.largePadding)
            )
            LoginContent(uiState, onAccountNumberChange, onLoginClick, onDeleteHistoryClick)
            Spacer(modifier = Modifier.weight(3f))
            CreateAccountPanel(onCreateAccountClick, isEnabled = uiState.loginState is Idle)
        }
    }
}

@Composable
@OptIn(ExperimentalComposeUiApi::class)
private fun LoginContent(
    uiState: LoginUiState,
    onAccountNumberChange: (String) -> Unit,
    onLoginClick: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.sideMargin)) {
        Text(
            text = uiState.loginState.title(),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier =
                Modifier.testTag(LOGIN_TITLE_TEST_TAG)
                    .fillMaxWidth()
                    .padding(bottom = Dimens.smallPadding)
        )

        var tfFocusState: FocusState? by remember { mutableStateOf(null) }
        var ddFocusState: FocusState? by remember { mutableStateOf(null) }
        val expandedDropdown = tfFocusState?.hasFocus ?: false || ddFocusState?.hasFocus ?: false

        Text(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            text = uiState.loginState.supportingText() ?: "",
            style = MaterialTheme.typography.labelMedium,
            color =
                if (uiState.loginState.isError()) {
                    MaterialTheme.colorScheme.error
                } else {
                    MaterialTheme.colorScheme.onPrimary
                },
        )

        TextField(
            modifier =
                // Fix for DPad navigation
                Modifier.onFocusChanged { tfFocusState = it }
                    .focusProperties {
                        left = FocusRequester.Cancel
                        right = FocusRequester.Cancel
                    }
                    .fillMaxWidth(),
            value = uiState.accountNumberInput,
            label = {
                Text(
                    text = stringResource(id = R.string.login_description),
                    color = Color.Unspecified
                )
            },
            keyboardActions =
                KeyboardActions(onDone = { onLoginClick(uiState.accountNumberInput) }),
            keyboardOptions =
                KeyboardOptions(
                    imeAction = if (uiState.loginButtonEnabled) ImeAction.Done else ImeAction.None,
                    keyboardType = KeyboardType.NumberPassword
                ),
            onValueChange = onAccountNumberChange,
            singleLine = true,
            maxLines = 1,
            visualTransformation = accountTokenVisualTransformation(),
            enabled = uiState.loginState is Idle,
            colors = mullvadWhiteTextFieldColors(),
            isError = uiState.loginState.isError(),
        )

        AnimatedVisibility(visible = uiState.lastUsedAccount != null && expandedDropdown) {
            val token = uiState.lastUsedAccount?.value.orEmpty()
            val accountTransformation = remember { accountTokenVisualTransformation() }
            val transformedText =
                remember(token) { accountTransformation.filter(AnnotatedString(token)).text }

            AccountDropDownItem(
                modifier = Modifier.onFocusChanged { ddFocusState = it },
                accountToken = transformedText.toString(),
                onClick = {
                    uiState.lastUsedAccount?.let {
                        onAccountNumberChange(it.value)
                        onLoginClick(it.value)
                    }
                },
                onDeleteClick = onDeleteHistoryClick
            )
        }

        Spacer(modifier = Modifier.size(Dimens.largePadding))
        VariantButton(
            isEnabled = uiState.loginButtonEnabled,
            onClick = { onLoginClick(uiState.accountNumberInput) },
            text = stringResource(id = R.string.login_title),
            modifier = Modifier.padding(bottom = Dimens.mediumPadding)
        )
    }
}

@Composable
private fun LoginIcon(loginState: LoginState, modifier: Modifier = Modifier) {
    Box(
        contentAlignment = Alignment.Center,
        modifier = modifier.size(Dimens.loginIconContainerSize)
    ) {
        when (loginState) {
            is Idle ->
                if (loginState.loginError != null) {
                    Image(
                        painter = painterResource(id = R.drawable.icon_fail),
                        contentDescription = stringResource(id = R.string.login_fail_title),
                        contentScale = ContentScale.Inside
                    )
                } else {
                    // If view is Idle, we display empty box to keep the same size as other states
                }
            is Loading ->
                CircularProgressIndicator(
                    modifier = Modifier.fillMaxSize(),
                    color = MaterialTheme.colorScheme.onPrimary,
                    strokeWidth = Dimens.loadingSpinnerStrokeWidth,
                    strokeCap = StrokeCap.Round
                )
            Success ->
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
                is Idle ->
                    when (this.loginError) {
                        is LoginError -> R.string.login_fail_title
                        null -> R.string.login_title
                    }
                is Loading -> R.string.logging_in_title
                Success -> R.string.logged_in_title
            }
    )

@Composable
private fun LoginState.supportingText(): String? {
    val res =
        when (this) {
            is Idle -> {
                when (loginError) {
                    LoginError.InvalidCredentials -> R.string.login_fail_description
                    LoginError.UnableToCreateAccount -> R.string.failed_to_create_account
                    is LoginError.Unknown -> R.string.error_occurred
                    null -> return null
                }
            }
            is Loading.CreatingAccount -> R.string.creating_new_account
            is Loading.LoggingIn,
            Success -> R.string.logging_in_description
        }
    return stringResource(id = res)
}

@Composable
private fun AccountDropDownItem(
    modifier: Modifier = Modifier,
    accountToken: String,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit
) {
    Row(
        modifier =
            modifier
                .clip(
                    MaterialTheme.shapes.medium.copy(
                        topStart = CornerSize(0f),
                        topEnd = CornerSize(0f)
                    )
                )
                .background(MaterialTheme.colorScheme.background)
                .height(IntrinsicSize.Min),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Box(
            modifier =
                Modifier.clickable(onClick = onClick)
                    .fillMaxHeight()
                    .weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
            contentAlignment = Alignment.CenterStart
        ) {
            Text(text = accountToken, overflow = TextOverflow.Clip)
        }
        IconButton(onClick = onDeleteClick) {
            Icon(
                painter = painterResource(id = R.drawable.account_history_remove_pressed),
                contentDescription = null,
                modifier = Modifier.size(Dimens.listIconSize),
                tint = Color.Unspecified
            )
        }
    }
}

@Composable
private fun CreateAccountPanel(onCreateAccountClick: () -> Unit, isEnabled: Boolean) {
    Column(
        Modifier.fillMaxWidth()
            .background(MaterialTheme.colorScheme.background)
            .padding(horizontal = Dimens.sideMargin, vertical = Dimens.screenVerticalMargin),
    ) {
        Text(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            text = stringResource(id = R.string.dont_have_an_account),
            color = MaterialTheme.colorScheme.onPrimary,
        )
        PrimaryButton(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.create_account),
            isEnabled = isEnabled,
            onClick = onCreateAccountClick
        )
    }
}
