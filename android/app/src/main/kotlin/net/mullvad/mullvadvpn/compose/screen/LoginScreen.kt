package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.window.PopupProperties
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithTopBar
import net.mullvad.mullvadvpn.compose.state.LoginError
import net.mullvad.mullvadvpn.compose.state.LoginState
import net.mullvad.mullvadvpn.compose.state.LoginState.*
import net.mullvad.mullvadvpn.compose.state.LoginUiState
import net.mullvad.mullvadvpn.compose.util.accountTokenVisualTransformation
import net.mullvad.mullvadvpn.lib.theme.AlphaTopBar
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewIdle() {
    AppTheme { LoginScreen(state = LoginUiState()) }
}

@Preview
@Composable
private fun PreviewLoggingIn() {
    AppTheme { LoginScreen(state = LoginUiState(loginState = Loading.LoggingIn)) }
}

@Preview
@Composable
private fun PreviewCreatingAccount() {
    AppTheme { LoginScreen(state = LoginUiState(loginState = Loading.CreatingAccount)) }
}

@Preview
@Composable
private fun PreviewLoginError() {
    AppTheme { LoginScreen(state = LoginUiState(loginState = Idle(LoginError.InvalidCredentials))) }
}

@Preview
@Composable
private fun PreviewLoginSuccess() {
    AppTheme { LoginScreen(state = LoginUiState(loginState = Success)) }
}

@Composable
fun LoginScreen(
    state: LoginUiState,
    onLoginClick: (String) -> Unit = {},
    onCreateAccountClick: () -> Unit = {},
    onDeleteHistoryClick: () -> Unit = {},
    onAccountNumberChange: (String) -> Unit = {},
    onSettingsClick: () -> Unit = {},
) {
    ScaffoldWithTopBar(
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
                state.loginState,
                modifier =
                    Modifier.align(Alignment.CenterHorizontally)
                        .padding(bottom = Dimens.largePadding)
            )
            LoginContent(state, onAccountNumberChange, onLoginClick, onDeleteHistoryClick)
            Spacer(modifier = Modifier.weight(3f))
            CreateAccountPanel(onCreateAccountClick, isEnabled = state.loginState is Idle)
        }
    }
}

@Composable
@OptIn(ExperimentalMaterial3Api::class)
private fun LoginContent(
    state: LoginUiState,
    onAccountNumberChange: (String) -> Unit,
    onLoginClick: (String) -> Unit,
    onDeleteHistoryClick: () -> Unit
) {
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.sideMargin)) {
        Text(
            text = state.loginState.title(),
            style = MaterialTheme.typography.headlineLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier.fillMaxWidth().padding(bottom = Dimens.smallPadding)
        )

        var expanded by remember { mutableStateOf(false) }

        Text(
            modifier = Modifier.padding(bottom = Dimens.smallPadding),
            text = state.loginState.supportingText() ?: "",
            style = MaterialTheme.typography.labelMedium,
            color =
                if (state.loginState.isError()) {
                    MaterialTheme.colorScheme.error
                } else {
                    MaterialTheme.colorScheme.onPrimary
                },
        )
        ExposedDropdownMenuBox(expanded = expanded, onExpandedChange = { expanded = it }) {
            TextField(
                modifier =
                    Modifier.then(
                            // Using menuAnchor while not showing a dropdown will cause keyboard to
                            // open and app to crash on navigation
                            if (state.lastUsedAccount != null) Modifier.menuAnchor() else Modifier
                        )
                        .fillMaxWidth(),
                value = state.accountNumberInput,
                label = {
                    Text(
                        text = stringResource(id = R.string.login_description),
                        color = Color.Unspecified
                    )
                },
                keyboardActions =
                    KeyboardActions(onDone = { onLoginClick(state.accountNumberInput) }),
                keyboardOptions =
                    KeyboardOptions(
                        imeAction =
                            if (state.loginButtonEnabled) ImeAction.Done else ImeAction.None,
                        keyboardType = KeyboardType.NumberPassword
                    ),
                onValueChange = onAccountNumberChange,
                singleLine = true,
                maxLines = 1,
                visualTransformation = accountTokenVisualTransformation(),
                enabled = state.loginState is Idle,
                colors =
                    TextFieldDefaults.colors(
                        focusedTextColor = Color.Black,
                        unfocusedTextColor = Color.Gray,
                        disabledTextColor = Color.Gray,
                        errorTextColor = Color.Black,
                        cursorColor = MaterialTheme.colorScheme.background,
                        focusedPlaceholderColor = MaterialTheme.colorScheme.background,
                        unfocusedPlaceholderColor = MaterialTheme.colorScheme.primary,
                        focusedLabelColor = MaterialTheme.colorScheme.background,
                        disabledLabelColor = Color.Gray,
                        unfocusedLabelColor = MaterialTheme.colorScheme.background,
                        focusedLeadingIconColor = Color.Black,
                        unfocusedSupportingTextColor = Color.Black,
                    ),
                isError = state.loginState.isError(),
            )

            // If we have a previous account, show dropdown for quick re-login
            state.lastUsedAccount?.let { token ->
                DropdownMenu(
                    modifier =
                        Modifier.background(MaterialTheme.colorScheme.background)
                            .exposedDropdownSize(true),
                    expanded = expanded,
                    properties = PopupProperties(focusable = false),
                    onDismissRequest = { expanded = false }
                ) {
                    val accountTransformation = remember { accountTokenVisualTransformation() }
                    val transformedText =
                        remember(token.value) {
                            accountTransformation.filter(AnnotatedString(token.value)).text
                        }

                    AccountDropDownItem(
                        accountToken = transformedText.toString(),
                        onClick = {
                            onAccountNumberChange(token.value)
                            expanded = false
                            onLoginClick(token.value)
                        }
                    ) {
                        onDeleteHistoryClick()
                    }
                }
            }
        }

        Spacer(modifier = Modifier.size(Dimens.largePadding))
        ActionButton(
            isEnabled = state.loginButtonEnabled,
            onClick = { onLoginClick(state.accountNumberInput) },
            colors =
                ButtonDefaults.buttonColors(
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    containerColor = MaterialTheme.colorScheme.surface
                ),
            text = stringResource(id = R.string.login_title),
            modifier = Modifier.padding(bottom = Dimens.mediumPadding)
        )
    }
}

@Composable
private fun LoginIcon(state: LoginState, modifier: Modifier = Modifier) {
    Box(
        contentAlignment = Alignment.Center,
        modifier = modifier.size(Dimens.loginIconContainerSize)
    ) {
        when (state) {
            is Idle ->
                if (state.loginError != null) {
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
                    modifier = Modifier.size(Dimens.progressIndicatorSize),
                    color = MaterialTheme.colorScheme.onPrimary,
                    strokeWidth = Dimens.loadingSpinnerStrokeWidth,
                    strokeCap = StrokeCap.Round
                )
            Success ->
                Image(
                    modifier = Modifier.offset(-Dimens.smallPadding, -Dimens.smallPadding),
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
    accountToken: String,
    onClick: () -> Unit,
    onDeleteClick: () -> Unit
) {
    Row(
        modifier = Modifier.clickable(onClick = onClick),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            modifier =
                Modifier.weight(1f)
                    .padding(horizontal = Dimens.mediumPadding, vertical = Dimens.smallPadding),
            text = accountToken
        )
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
        ActionButton(
            modifier = Modifier.fillMaxWidth(),
            text = stringResource(id = R.string.create_account),
            isEnabled = isEnabled,
            colors =
                ButtonDefaults.buttonColors(
                    contentColor = MaterialTheme.colorScheme.onPrimary,
                    containerColor = MaterialTheme.colorScheme.primary
                ),
            onClick = onCreateAccountClick
        )
    }
}
